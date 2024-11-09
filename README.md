# Rust 语言
#nginx 容器部署使用docker-compose.yaml
命令：docker compose -f docker-compse.yaml -p nginx up -d

启用容器之前需要先获取默认nginx.conf文件，使用以下命令
$ docker run --rm --entrypoint=cat nginx /etc/nginx/nginx.conf > ./volume/nginx/nginx.conf

nginx的配置修改，可以在/volume/nginx/nginx.conf 这个目录下进行修改
