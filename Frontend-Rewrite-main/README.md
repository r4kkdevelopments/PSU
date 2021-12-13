# PSU Frontend

The new rewrite for the PSU frontend.

# How to Run
How to run the PSU frontend rewrite in development mode, or production mode.

# Production
### Pre-requisites
- A server with Docker installed
- A deploy key (see [here](https://docs.github.com/en/developers/overview/managing-deploy-keys#deploy-keys) and [here, for how to clone repositories using said key](https://gist.github.com/zhujunsan/a0becf82ade50ed06115))
- Nginx installation setup on the server

### Nginx Configuration (if not setup already)
```nginx
server {
    listen 80;
    listen [::]:80;
    
    server_name domain www.domain;
    
    location / {
        proxy_pass http://localhost:3000;
    }
}
```
Should you need SSL, there are tutorials for setting up SSL with Certbot or even with a custom certificate. Use Google.

### Steps
- Firstly, clone the Git repository on the server using your deploy key.
- Then, run `docker build . -t psu-frontend`
- Lastly, run `docker run -d -p 3000:3000 psu-frontend`

### Other Information
- To check up on the container and check if it's still running, simply run `docker ps`.

# Development
### Pre-requisites
- Node 15.x
- NPM

### Steps
- Clone this Git repository on your local machine
- Navigate into the repository and run `npm i`
- Once that's completed, you should be able to just do `npm run dev`.
- After which, have fun developing.

# Licensing
This project is licensed under GPL-3.0, however I will add the licensing later on.

[![Deploy to Server](https://github.com/psuDevelopment/Frontend-Rewrite/actions/workflows/deploy.yml/badge.svg)](https://github.com/psuDevelopment/Frontend-Rewrite/actions/workflows/deploy.yml)
