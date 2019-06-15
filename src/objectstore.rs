use std::io::{stdout, Read, Write};
use std::fs::File;

use indicatif::{ProgressBar, ProgressStyle};
use curl::easy::{Easy, List};
use memmap::MmapOptions;
use serde_json::Value as JSONValue;


use error::OpenstackError;
use client::Response;

pub fn upload_to_object_store(
    file: &mut File,
    object_store_url: &str,
    token: &str,
) -> Result<Response, OpenstackError> {
    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;

    let mut data = Vec::new();

    let mut easy = Easy::new();
    easy.url(object_store_url)?;
    easy.upload(true)?;
    easy.http_headers(headers)?;
    easy.progress(true)?;

    let mut remote_headers = vec![];

    {
        let progress_bar = make_progress_bar(file.metadata()?.len());
        let mut transfer = easy.transfer();
        transfer.progress_function(|_a, _b, _c, d| {
            if d as u32 != 0 {
                progress_bar.set_position(d as u64);
            }
            true
        })?;
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
        transfer.header_function(|header| {
            remote_headers.push(String::from_utf8(header.to_vec()).unwrap_or(String::from(" : ")));
            true
        })?;
        transfer.read_function(|into| Ok(file.read(into).unwrap()))?;
        transfer.perform()?;
        progress_bar.finish();
    }

    // println!("{:?}", String::from_utf8(data).unwrap_or(String::from("")));
    // Ok(String::from_utf8(data).unwrap_or(String::from("")))
    let response_data = JSONValue::String(String::from_utf8(data).unwrap_or(String::from("")));
    Ok(Response{0: response_data, 1: easy.response_code()?, 2: remote_headers})

}

pub fn download_from_object_store(
    file: &mut File,
    object_store_url: &str,
    token: &str,
) -> Result<Response, OpenstackError> {
    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;

    // let urlpath = &format!("{}/{}/{}", object_store_url, container, name);
    let urlpath = object_store_url;

    let mut easy = Easy::new();
    easy.url(urlpath)?;
    easy.get(true)?;
    easy.http_headers(headers)?;
    easy.progress(true)?;

    let mut remote_headers = vec![];

    {
        let progress_bar = make_progress_bar(0);
        let mut transfer = easy.transfer();
        transfer.progress_function(|a, b, _c, _d| {
            if a as u32 != 0 {
                progress_bar.set_length(a as u64);
                progress_bar.set_position(b as u64);
            }
            true
        })?;
        transfer.write_function(|data| Ok(file.write(data).unwrap()))?;
        transfer.header_function(|header| {
            remote_headers.push(String::from_utf8(header.to_vec()).unwrap_or(String::from(" : ")));
            true
        })?;
        transfer.perform()?;
        progress_bar.finish();
    }

    let _statuscode = easy.response_code()?;

    // Ok((String::from(""), statuscode))
    let response_data = JSONValue::String(String::from(""));
    Ok(Response{0: response_data, 1: easy.response_code()?, 2: remote_headers})
}


pub fn upload_to_object_store_dynamic_large_objects(
    file: &File,
    name: &str,
    container: &str,
    object_store_url: &str,
    token: &str,
    parts: usize,
    skip_first: usize,
) -> Result<Response, OpenstackError> {
    let fileurl = object_store_url;

    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;

    let mut easy = Easy::new();
    easy.url(fileurl)?;
    easy.upload(true)?;
    easy.http_headers(headers)?;
    easy.write_function(|into| Ok(stdout().write(into).unwrap()))?;
    easy.perform()?;

    let mut responses = Vec::new();
    let mut success = true;
    {
        let mmap = unsafe { MmapOptions::new().map(file)? };
        let amount = (mmap.len() as f32 / (parts as f32 - 0.1)) as usize;

        let chuckies = mmap.chunks(amount).skip(skip_first);
        for (index, mut chunk) in chuckies.enumerate() {
            let response = upload_part(&mut chunk, index, fileurl, token)?;

            if !response.is_success() {
                success = false;
                responses.push(response);
                break;
            }
            responses.push(response);

        }
    }
    if success == true {
        set_dynamic_manifest(fileurl, container, name, token)?;
    }

    match success {
        true => Ok(responses[responses.len()-1].clone()),
        false => Ok(responses[0].clone()),
    }
}

pub fn open_file(filename: &str) -> Result<File, OpenstackError> {
    let filepath = is_file(filename)?;
    Ok(File::open(filepath)?)
}

pub fn create_file(filename: &str) -> Result<File, OpenstackError> {
    make_sure_folder_exists(filename)?;
    Ok(File::create(filename)?)
}

fn is_file(filename: &str) -> Result<std::path::PathBuf, OpenstackError> {
    let filepath = std::path::PathBuf::from(filename);
    if !filepath.is_file() {
        return Err(
            OpenstackError::new(&format!("'{}' does not exist", filename))
        );
    }
    Ok(filepath)
}

fn make_sure_folder_exists(filename: &str) -> Result<(), OpenstackError> {
    let filepath = std::path::PathBuf::from(filename);
    let parent = match filepath.parent(){
        Some(x) => x.to_path_buf(),
        None => return Ok(())
    };
    std::fs::DirBuilder::new().recursive(true).create(parent)?;
    Ok(())
}

fn make_objectstore_url(store_url: &str, container: &str, filename: &str) -> String {
    format!("{}/{}/{}", store_url, container, filename)
}

fn set_dynamic_manifest(
    fileurl: &str,
    container: &str,
    filename: &str,
    token: &str,
) -> Result<(String, u32), OpenstackError> {
    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;
    headers.append(&format!("X-Object-Manifest: {}/{}/", container, filename))?;

    let mut data = Vec::new();
    let mut easy = Easy::new();
    easy.url(&fileurl)?;
    easy.upload(true)?;
    easy.http_headers(headers)?;
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }
    Ok((
        String::from_utf8(data).unwrap_or(String::from("")),
        easy.response_code()?,
    ))
}

fn upload_part(
    chunk: &mut &[u8],
    index: usize,
    fileurl: &str,
    token: &str,
) -> Result<Response, OpenstackError> {
    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;

    let mut data = Vec::new();
    let mut easy = Easy::new();
    easy.url(&format!("{}/{:08}", fileurl, index))?;
    easy.upload(true)?;
    easy.progress(true)?;
    easy.http_headers(headers)?;

    let mut remote_headers = vec![];

    {
        let progress_bar = make_progress_bar(chunk.len() as u64);
        let mut transfer = easy.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.progress_function(|_a, _b, _c, d| {
            if d as u32 != 0 {
                progress_bar.set_position(d as u64);
            }
            true
        })?;
        transfer.header_function(|header| {
            remote_headers.push(String::from_utf8(header.to_vec()).unwrap_or(String::from(" : ")));
            true
        })?;

        transfer.read_function(|into| Ok(chunk.read(into).unwrap()))?;
        transfer.perform()?;
        progress_bar.finish();
    }

    Ok(Response{
        0:  JSONValue::String(String::from_utf8(data).unwrap_or(String::from(""))),
        1: easy.response_code()?,
        2: remote_headers,
    })
}

pub fn make_progress_bar(length: u64) -> ProgressBar{
    let progress_bar = ProgressBar::new(length);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {wide_bar:.cyan/blue} {bytes:>7}/{total_bytes:7}"),
    );
    progress_bar
}