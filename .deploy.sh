#! /bin/bash

RUST_VERSION=$(rustc --version | sed 's/rustc //' | sed 's/(.*)//' | tr -d '[:space:]')
RUST_TOOLCHAIN=$(rustup show active-toolchain | sed 's/(default)//' | tr -d '[:space:]')
CONTAINER="rustci"

echo "$RUST_VERSION"
echo "$RUST_TOOLCHAIN"

if [ "$TRAVIS_OS_NAME" == "linux" ]; then
    FILE=/home/travis/build/thomas9911/openstack-client/target/release/openstack-client
    CLIENT_VERSION=$($FILE --version | sed 's/openstack-client//' | tr -d '[:space:]')
    echo "$RUST_VERSION"
    echo "$RUST_TOOLCHAIN"
    echo "$CLIENT_VERSION"
    strip "$FILE"
    chmod +x "$FILE"
    $FILE upload object --file "$FILE" --name "$CLIENT_VERSION"/"$TRAVIS_OS_NAME"/"$RUST_TOOLCHAIN"/"$RUST_VERSION"/openstack-client --container "$CONTAINER"
    $FILE upload object --file "$FILE" --name latest/"$TRAVIS_OS_NAME"/openstack-client --container "$CONTAINER"
fi

if [ "$TRAVIS_OS_NAME" == "osx" ]; then
    FILE=/Users/travis/build/thomas9911/openstack-client/target/release/openstack-client
    CLIENT_VERSION=$($FILE --version | sed 's/openstack-client//' | tr -d '[:space:]')
    echo "$RUST_VERSION"
    echo "$RUST_TOOLCHAIN"
    strip "$FILE"
    chmod +x "$FILE"
    $FILE upload object --file "$FILE" --name "$CLIENT_VERSION"/"$TRAVIS_OS_NAME"/"$RUST_TOOLCHAIN"/"$RUST_VERSION"/openstack-client --container "$CONTAINER"
    $FILE upload object --file "$FILE" --name latest/"$TRAVIS_OS_NAME"/openstack-client --container "$CONTAINER"
fi

if [ "$TRAVIS_OS_NAME" == "windows" ]; then
    FILE=/c/Users/travis/build/thomas9911/openstack-client/target/release/openstack-client.exe
    CLIENT_VERSION=$($FILE --version | sed 's/openstack-client//' | tr -d '[:space:]')
    echo "$RUST_VERSION"
    echo "$RUST_TOOLCHAIN"
    $FILE upload object --file "$FILE" --name "$CLIENT_VERSION"/"$TRAVIS_OS_NAME"/"$RUST_TOOLCHAIN"/"$RUST_VERSION"/openstack-client.exe --container "$CONTAINER"
    $FILE upload object --file "$FILE" --name latest/"$TRAVIS_OS_NAME"/openstack-client.exe --container "$CONTAINER"
fi