# Cryptifier

Fetches...

1. Price of crypto currencies from Coin Gecko
2. CBBI from colintalkscrypto.com
3. Rainbow chart from blockchaincenter.net
4. Carbon Emissions Futures from investing.com

Notifies users via...

1. Telegram

## Setup

Prerequisites:

- Node.js

An easy way to setup Node is to use `nvm`. Then run `nvm use` to get the proper version specified in the project.

Then install the dependencies...

```
$ npm ci
```

## Development

Useful commands:

`$ npm run dev` - This will build images, scripts and styles and also watch changes in the two latter.

## Testing

Tests are written in a BDD/Cucumber type syntax using `mocha-cakes-2`. Try to write tests in a meaningful way as to describe what it is you're testing and what resources are available. Testing does not only test a piece of code that it actually works but is also used for documentation purposes. Focus on testing what is vital for the feature.

To run all tests (including linting): `$ npm test`
