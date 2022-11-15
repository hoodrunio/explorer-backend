# Explorer Backend

<p align="center">
  <a href="/"><img src="https://external-content.duckduckgo.com/iu/?u=http%3A%2F%2Fupload.wikimedia.org%2Fwikipedia%2Fcommons%2Fthumb%2Fd%2Fd7%2FDesktop_computer_clipart_-_Yellow_theme.svg%2F640px-Desktop_computer_clipart_-_Yellow_theme.svg.png" alt="Logo" height=220>
  </a>
</p>


Explorer backend is an app:
- Deals with blockchain nodes.
- Supports our database implementation.
- Provides a nice REST & websocket API.

Our backend is the fastest explorer backend possible, and it makes our website the best Cosmos explorer.

**It isn't completed yet.**


## Development
Install Rust language toolkit.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Run the backend server
```bash
cargo run
```


## Undone Things
- **APR** calculation.
- Parsing some data for transactions and blocks at
[`src/fetch/wss/new_blocks.rs`](https://github.com/testnetrunn/explorer-backend/blob/main/src/fetch/wss/new_blocks.rs), and
[`src/fetch/wss/tx.rs`](https://github.com/testnetrunn/explorer-backend/blob/main/src/fetch/wss/tx.rs). ✅ HALFLY DONE
- **Validators**. To be added via creating a new file inside 
[`src/fetch/wss/`](https://github.com/testnetrunn/explorer-backend/blob/main/src/fetch/wss/).
- **Proposals**. To be added via creating a new file inside 
[`src/fetch/wss/`](https://github.com/testnetrunn/explorer-backend/blob/main/src/fetch/wss).
- **Params**. ✅ DONE
- Create a nice interface to interact with Web Socket endpoint.




## Usage
Test the REST API
- Go to `src/routes/rest` folder.
- Check the available paths.
- Test them by visiting with a browser.

Test Web Sockets
- Create a new Web Socket connection to `ws://localhost:8000/{chain}/socket`, where `{chain}` is your preferred chain.
- There are two modes currently. One is `blocks`, and the other is `txs`.

### Example
Open your browsers console by pressing `fn + f12`.
And paste the code below.
```js
// Create a Web Socket connection.
const ws = new WebSocket('ws://localhost:8000/axelar/socket');

// Add an `open` event listener.
ws.addEventListener('open', () => {
  console.log('Web Socket connection is established!');
});

// Add an `message` event listener.
ws.addEventListener('message', (e) => {
  console.log('DATA FROM SERVER:');
  console.log(e.data)
});


// Add an `close` event listener.
ws.addEventListener('close', (e) => {
  console.log('Connection is closed!')
});

// Available options are "blocks", "tx", "params".
setTimeout(() => ws.send("blocks"), 1000)
// Then it starts listening "blocks"
```
> The data from the backend arrives as JSON encoded `string`.

> So you have to parse it before accessing its properties.
