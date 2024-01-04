build:
	docker build -t 3d-renderer-2stage .

upload-gltf:
	aws s3 cp ./glb/ s3://jq-staging-matko/gltf/ --recursive --profile jq-staging-sysops

run-server: build
	docker run --init --memory=1024m --cpus=1 -it --rm -p 3030:3030 3d-renderer-2stage

vegeta:
	vegeta attack -targets=request.txt -format=http -duration=30s -rate=5 \
	| tee results.bin \
	| vegeta report
