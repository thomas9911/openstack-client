RUST_VERSION=$(rustc --version | sed 's/rustc //' | sed 's/(.*)//' | tr -d [:space:])
RUST_TOOLCHAIN=$(rustup show active-toolchain | sed 's/(default)//' | tr -d [:space:])
echo $RUST_VERSION
echo $RUST_TOOLCHAIN

if [ "$TRAVIS_OS_NAME" == "linux" ]; then
    FILE=/home/travis/build/thomas9911/openstack-client/target/release/openstack-client
    CLIENT_VERSION=$($FILE --version | sed 's/openstack-client//' | tr -d [:space:])
    ls
    pwd
    echo $RUST_VERSION
    echo $RUST_TOOLCHAIN
    echo $FILE
    echo $CLIENT_VERSION
    strip "$FILE"
    $FILE upload object --file "$FILE" --name $CLIENT_VERSION/$TRAVIS_OS_NAME/$RUST_TOOLCHAIN/$RUST_VERSION/openstack-client --container rustci
fi

if [ "$TRAVIS_OS_NAME" == "osx" ]; then
    ls
    pwd
    echo $RUST_VERSION
    echo $RUST_TOOLCHAIN
    # strip /home/travis/build/thomas9911/openstack-client/target/release/openstack-client
    # /home/travis/build/thomas9911/openstack-client/target/release/openstack-client upload object --file /home/travis/build/thomas9911/openstack-client/target/release/openstack-client --name $CLIENT_VERSION/$TRAVIS_OS_NAME/$RUST_TOOLCHAIN/$RUST_VERSION/openstack-client --container rustci
fi