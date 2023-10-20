+++
title = "Docker image"
weight = 90
+++

If you have to distribute a Zola based web site through Docker, it's easy to do with a multi-stage build.

Here is an example that builds the current folder, and put the result in a docker image that will be served by
[static-web-server](https://static-web-server.net/), a minimalist web server written in rust.

Of course, you may want to replace the second stage with another static web server like Nginx or Apache.

```Dockerfile
FROM ghcr.io/getzola/zola:v0.17.1 as zola

COPY . /project
WORKDIR /project
RUN ["zola", "build"]

FROM ghcr.io/static-web-server/static-web-server:2
WORKDIR /
COPY --from=zola /project/public /public
```

To build your website as a docker image, you then run:
```shell
docker build -t my_website:latest .
```

To test your site, just run the docker image and browse [http://localhost:8000](http://localhost:8000)

```
docker run --rm -p 8000:80 my_website:latest
```

Note that, if you want to be able to use your docker image from multiple locations, you'll have to set `base_url` to `/`.
