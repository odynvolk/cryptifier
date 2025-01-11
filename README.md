# Cryptifier

Fetches...

1. Price of crypto currencies from [https://www.coingecko.com/](https://colintalkscrypto.com/)
2. Number of reachable Bitcoin nodes from [https://bitnodes.io/](https://bitnodes.io/)
3. CBBI from [https://colintalkscrypto.com/](https://colintalkscrypto.com/)
3. Fear and Greed index from [https://alternative.me/](https://alternative.me/)
4. Carbon Emissions Futures from [https://sandbag.be/](https://sandbag.be/)

Notifies users of price going up or down in increments via...

1. Telegram via a TelegramBot

## Setup

Prerequisites:

- Node.js

An easy way to setup Node is to use either...

1) [asdf](https://asdf-vm.com/) Then run `asdf install` to get the proper version specified in the project.
2) [nvm](https://github.com/nvm-sh/nvm) Then run `nvm use` to get the proper version specified in the project.

Then install the dependencies...

```bash
$ npm ci
```

Configuration:

Create a .env file with values needed in your setup.

```
logLevel=info|debug
currencies=[{"ticker": "bitcoin","increment": 1000}]
telegramApiKey=secret-key-for-bot
telegramChatIds=some-id,some-other-id
```

## Development

Useful commands:

`$ npm run dev` - This will build images, scripts and styles and also watch changes in the two latter.

## Testing

Tests are written in a BDD/Cucumber type syntax using `mocha-cakes-2`. Try to write tests in a meaningful way as to
describe what it is you're testing and what resources are available. Testing does not only test a piece of code that it
actually works but is also used for documentation purposes. Focus on testing what is vital for the feature.

To run all tests (including linting)...

```bash
$ npm test
```
