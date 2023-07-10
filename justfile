# build the dockerfile
docker-build:
    docker build . \
        --build-arg ancalagon_index="${ANCALAGON_INDEX}" \
        --build-arg ancalagon_token="${ANCALAGON_TOKEN}"

# start test loop with cargo watch
watch:
    cargo watch -x test
