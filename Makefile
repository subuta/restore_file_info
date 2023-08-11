# For testing S3 cache in local.
.PHONY: build_with_s3_cache
build_with_s3_cache:
	cd ./build && _EXPERIMENTAL_DAGGER_CACHE_CONFIG='type=s3,mode=max,endpoint_url=http://host.docker.internal:9000,access_key_id=minio_user,secret_access_key=minio_password,region=us-east-1,use_path_style=true,bucket=dagger-s3-cache' yarn build

# Preapre local minio bucket.
.PHONY: prepare-minio
prepare-minio:
	docker run --rm --network="restore_file_info_default" --entrypoint sh minio/mc -c "mc config host add myminio http://minio:9000 minio_user minio_password && mc mb --ignore-existing myminio/dagger-s3-cache && mc anonymous set none myminio/dagger-s3-cache"

# Build for all targets in local
.PHONY: build
build:
	cd ./build && yarn build -t x86_64-unknown-linux-musl && yarn build -t aarch64-unknown-linux-musl
