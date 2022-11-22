# Explorer Backend

<p align="center">
  <a href="/"><img src="https://external-content.duckduckgo.com/iu/?u=http%3A%2F%2Fupload.wikimedia.org%2Fwikipedia%2Fcommons%2Fthumb%2Fd%2Fd7%2FDesktop_computer_clipart_-_Yellow_theme.svg%2F640px-Desktop_computer_clipart_-_Yellow_theme.svg.png" alt="Logo" height=220>
  </a>
</p>


**Explorer backend is an app:**
- Deals with blockchain nodes.
- Supports our database implementation.
- Provides a nice REST & websocket API.


# To-do
- **APR** calculation.
- **Indexing** special chain specific data.
- A great **database** implementation to store important stuff.
- A nice **WebSocket** interface to provide multiple events to subscribe dynamic data.



# Usage
> If you don't have Rust installed, follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

- Clone the repository by typing this in terminal: `git clone https://github.com/testnetrunn/explorer-backend.git`
- Set working directory to the project by typing this in terminal: `cd explorer-backend`
- Run the project by typing this in terminal: `cargo run --release`
- Go to [`src/routes/`](https://github.com/testnetrunn/explorer-backend/tree/main/src/routes) folder, and pick any of the files inside.
- Each function represents a different path.
- Test by visiting paths with a browser.

# Development




## Support a new chain.

Open [`Chains.yaml`](https://github.com/testnetrunn/explorer-backend/blob/main/Chains.yml) file, and provide the required info below in the end of the file:
- Name
- Logo
- RPC URL
- REST API URL
- Web Socket URL
> All other info is optional, and some of them are generated automatically.

For example: 
```yaml
•••

name: axelar
logo: https://assets.coingecko.com/coins/images/24489/large/tsYr25vB_400x400.jpg
rpc: https://rpc.cosmos.directory/axelar
rest: https://axelar-api.polkachu.com
wss: wss://axelar-rpc.chainode.tech/websocket
```




## Support a new endpoint.

Go to [`src/routes`](https://github.com/testnetrunn/explorer-backend/tree/main/src/routes), and find or create a new file for the category associated with the new endpoint to be supported.

> Don't forget to import important stuff.

Create a new function representing the endpoint like below:
```rs
#[get("{chain}/example")]
pub async fn example(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_example().await.into(),
        Err(err) => Response::Error(err),
    })
}
```
 `{chain}` means a path variable, and its type is defined at `Path<String>`.

 If there is also another variable, we define its type as `Path<(String, OtherVarType)>`.
 
 > You have to create a fetch method for `Chain` struct before creating the endpoint.
 
 ## Create a new method.
 
Go to [`src/fetch`](https://github.com/testnetrunn/explorer-backend/tree/main/src/fetch), and find or create a new file for the category associated with the new method to be created.
 Define a new method inside `impl` block like below:
 ```rs
 impl Chain {
     •••
     
     pub async fn get_signing_info(&self, cons_addr: &str) -> Result<OutRestResponse<InternalSlashingSigningInfoItem>, String> {
        let path = format!("/cosmos/slashing/v1beta1/signing_infos/{cons_addr}");

        let resp = self.rest_api_request::<SigningInfoResp>(&path, &[]).await?;

        let signing_info = resp.val_signing_info.try_into()?;

        OutRestResponse::new(signing_info, 0)
    }
 
 }
 ```
 
 > If the method is not chain agnostic, you use a logic like `if self.inner.name == "evmos"` there.
