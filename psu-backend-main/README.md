# psu-backend
Fork of IOnic's PSU Backend Written in Rust


## Quick Start

Compiling this code requires you use RUSTC Nightly which you can switch to by running:
```shell
rustup default nightly
```

Next, the options for the backend will need to be added. Create the file ".env" in the root directory of the backend and copy the format below filling in the empty fields:

OPTIONAL - Backend will start without it
REQUIRED - Backend will not start without it

```
DATABASE_URL= Connection URL to postgreSQL **REQUIRED**

AWS_ACCESS_KEY_ID= AWS Access ID with S3 Read/Write permissions **OPTIONAL**
AWS_SECRET_ACCESS_KEY=  AWS Access key with S3 Read/Write permissions **OPTIONAL**
RUST_LOG=main

PAYPAL_ID= If using PayPal then add the PayPal ID **REQUIRED**
PAYPAL_SECRET= If using PayPal then add the PayPal Secret **REQUIRED**

STRIPE_KEY= Stripe Live Key **REQUIRED**
STIPE_WEBHOOK_KEY= Stripe Webhook Key **REQUIRED**

CAPTCHA_KEY= Google Captcha V2 Key **REQUIRED**

MAILGUN_USERNAME= Mailgun Username **OPTIONAL**
MAILGUN_PASSWORD= Mailgun Password **OPTIONAL**
MAILGUN_KEY= Mailgun API Key **REQUIRED**

DISCORD_ID= Discord Application Client ID **REQUIRED**
DISCORD_SECRET= Discord Application Secret **REQUIRED**
DISCORD_BOTTOKEN= Discord Application Bot Token **REQUIRED**
```

If you don't have a C compiler installed, install this to prevent errors (using any package manager):
```shell
apt install build-essential
```

Install the required dependencies:
```shell
apt install pkg-config libssl-dev
```

Once done, compile the code with:
```shell
cargo build --release
```

If you occur no errors and the application has compiled, navigate to (./target/release/) and add a file called: "Rocket.toml" with the following contents:
```toml
[global.databases]
postgres_db = { url = "POSTGRES CONNECTION URL" }

[global.limits]
forms = 52428800

```

Now that you have done that, in the ./target/release folder, you will find a file called "psu-backend" that is built to be executeable with your OS and simply just run it and the backend will start!
