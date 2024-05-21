<h1 align="center">leaveme</h1>

[![Screenshot](https://raw.githubusercontent.com/vaibhavpandeyvpz/leaveme/main/screenshot.png)](https://raw.githubusercontent.com/vaibhavpandeyvpz/leaveme/main/screenshot.png)

<p align="center">
Headless employee leave management in <a href="https://www.rust-lang.org/">Rust</a> built on <a href="https://slack.com/intl/en-in/">Slack</a> for small businesses.
Record leave requests, require manager approval and leave(s) history etc. provided as Slack slash command(s).
</p>

<p align="center">
<img alt="Build status" src="https://github.com/vaibhavpandeyvpz/leaveme/workflows/Release/badge.svg">
<img alt="GitHub Release" src="https://img.shields.io/github/v/release/vaibhavpandeyvpz/leaveme">
<img alt="GitHub Downloads (all assets, all releases)" src="https://img.shields.io/github/downloads/vaibhavpandeyvpz/leaveme/total">
</p>

## Usage

Grab a binary from the latest release for your platform from [this page](https://github.com/vaibhavpandeyvpz/leaveme/releases/latest).
In the same folder as binary, create a `config.yml` file from the sample in the repository using below command:

```shell
wget -O config.yml https://raw.githubusercontent.com/vaibhavpandeyvpz/leaveme/main/config.dist.yml
```

Then go to [api.slack.com](https://api.slack.com/), create a new app using provided manifest (see [slack.dist.yml](slack.dist.yml)) and install it on a [Slack](https://slack.com/intl/en-in/) workspace.
Once done, make note of the **signing secret** as well as **bot access token** shown in [Slack](https://slack.com/intl/en-in/).

Update your [Slack](https://slack.com/intl/en-in/) credentials in `config.yml` file and start the app server using below command:

```shell
./leaveme --config=config.yml
```

Since [Slack](https://slack.com/intl/en-in/) needs to communicate with your app for certain functionality, it's recommended to run this on a server and install an [SSL](https://letsencrypt.org/) certificate.

## Development

Make sure you have [Docker](https://www.docker.com/) installed on your workstation.
For the IDE, I highly recommend using [RustRover](https://www.jetbrains.com/rust/) i.e., my goto choice for [Rust](https://www.rust-lang.org/) development.

Download or clone the project using [Git](https://git-scm.com/) and then run following commands in project folder:

```shell
# create .env file in project
cp .env.dist .env

# update NGROK_AUTHTOKEN in .env

# create app config file
cp config.dist.yml config.yml

# update values in config.yml

# create ngrok config file
cp ngrok.dist.yml ngrok.yml

# update ngrok domain in ngrok.yml

# create Slack app manifest
cp slack.dist.yml slack.yml

# update ngrok domain in slack.yml

# start all services
docker compose up -d
```

## Deployment

For deployment, using a pre-built binary from [Releases](https://github.com/vaibhavpandeyvpz/leaveme/releases) section is the easiest way to go.

You could also use [Docker](https://www.docker.com/) for deployment. There's a bundled `Dockerfile` that builds and exposes the server on port `8000`.

To build the [Docker](https://www.docker.com/) container locally, use below command:

```shell
docker build -t leaveme .
# or 
docker build -t ghcr.io/vaibhavpandeyvpz/leaveme .
```

Container once pushed, can be pulled and run directly as below:

```shell
docker run -it --rm \
  -p "8000:8000" \
  -v ./config.yml:/leaveme_config.yml \
  ghcr.io/vaibhavpandeyvpz/leaveme:latest \
  leaveme --config=/leaveme_config.yml
```

You can also use below [Nginx](https://nginx.org/en/) vhost config to expose the server to internet easily:

```text
server {
    listen 80;
    listen [::]:80;

    server_name example.com;

    location ~ ^/leaveme/ {
        rewrite ^/leaveme/(.*) /$1 break;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_pass http://127.0.0.1:8000;
    }
}
```

If not running via [Docker](https://www.docker.com/), you can use below [Supervisor](https://supervisord.org/) configuration to run the server in daemon mode:

```ini
[program:leaveme]
autorestart=true
command=/home/ubuntu/leaveme --config=/home/ubuntu/leaveme_config.yml
autostart=true
autorestart=true
stopasgroup=true
killasgroup=true
redirect_stderr=true
stdout_logfile=/home/ubuntu/leaveme.log
stopwaitsecs=3600
```
