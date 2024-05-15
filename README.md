<h1 align="center">leaveme</h1>

<p align="center">
Headless employee leave management in <a href="https://www.rust-lang.org/">Rust</a> built on <a href="https://slack.com/intl/en-in/">Slack</a> for small businesses.
Record leave requests, require manager approval and leave(s) history etc. provided as slack workflows.
</p>

## Usage

Go to [api.slack.com](https://api.slack.com/), create a new app using provided manifest (see [slack.dist.yml](slack.dist.yml)) and install it on a [Slack](https://slack.com/intl/en-in/) workspace.
Once done, make note of **signing secret** as well as **bot access token** as you will need them later.

Grab a binary from the latest release for your platform from [this page](https://github.com/vaibhavpandeyvpz/leaveme/releases/latest).
In the same folder as binary, create a `.env` file from the sample in the repository using below command:

```shell
wget -O .env https://raw.githubusercontent.com/vaibhavpandeyvpz/leaveme/main/.env.dist
```

Update your [Slack](https://slack.com/intl/en-in/) credentials in `.env` file and start the app server using below command:

```shell
./leaveme
```

Since [Slack](https://slack.com/intl/en-in/) needs to communicate to your app for certain functionality, it's recommended to run this on a server and install an [SSL](https://letsencrypt.org/) certificate.

## Development

Make sure you have [Docker](https://www.docker.com/) installed on your workstation.
For the IDE, I highly recommend using [RustRover](https://www.jetbrains.com/rust/) i.e., my goto choice for [Rust](https://www.rust-lang.org/) development.

Download or clone the project using [Git](https://git-scm.com/) and then run following commands in project folder:

```shell
# create .env file in project
cp .env.dist .env

# update values e.g., NGROK_AUTHTOKEN in .env

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
```
