#
# docker factory
#
.PHONY = build run clean

build: clean
	@git clone https://github.com/Software-Knife-and-Tool/.git
	@docker build -f Dockerfile.build .
	@docker build -t ubuntu-thorn -f Dockerfile.ubuntu .
	@rm -rf thorn

run:
	docker run -it ubuntu-thorn /opt/thorn/bin/thorn

clean:
	@rm -rf thorn
