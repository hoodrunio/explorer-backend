# Explorer Backend

<p align="center">
  <a href="/"><img src="https://external-content.duckduckgo.com/iu/?u=http%3A%2F%2Fupload.wikimedia.org%2Fwikipedia%2Fcommons%2Fthumb%2Fd%2Fd7%2FDesktop_computer_clipart_-_Yellow_theme.svg%2F640px-Desktop_computer_clipart_-_Yellow_theme.svg.png" alt="Logo" height=220>
  </a>
</p>

This repo only serves as an API for the UI and has no D. connection with it.

The UI repo can be found [here.](https://github.com/testnetrunn/explorer-ui-v2)


**Explorer backend is an app:**
- Deals with blockchain nodes.
- Supports our database implementation.
- Provides REST & websocket API.


# To-do
- EVM TX support - decode ethabi func/specs. (✅)
- Smart contract verification (Solidity only) for Evmos. (✅: UI dependent works still in development.)
- Axelar EVM-poll/heartbeats features. (✅)
- **Database** implementation to store important stuff. (✅)
- **WebSocket** interface to provide multiple events to subscribe dynamic data. (✅)
- **gRPC** implementation. (gRPC infra instead of REST.) - Under development. (🚧)
- Support Kyve Pools and protocol layer features. - Under development. (🚧)



# Usage
> If you don't have Rust installed, follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

- Clone the repository 
```
git clone https://github.com/testnetrunn/explorer-backend.git
cd explorer-backend
```

- Install MongoDB
> Original resources can be found [here](https://www.mongodb.com/docs/manual/tutorial/install-mongodb-on-ubuntu).

```
wget -qO - https://www.mongodb.org/static/pgp/server-6.0.asc | sudo apt-key add -
echo "deb [ arch=amd64,arm64 ] https://repo.mongodb.org/apt/ubuntu focal/mongodb-org/6.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-6.0.list
sudo apt-get update
sudo apt-get install -y mongodb-org
sudo systemctl start mongod
```
- Run the project by typing this in terminal
```
cargo run --release
```
- Go to [`src/routes/`](https://github.com/testnetrunn/explorer-backend/tree/main/src/routes) folder, and pick any of the files inside.
- Each function represents a different path.
- Test by visiting paths with a browser.

For production, you might consider proxy. 
Here is an example for nginx:

```
server {
    listen 80;
    listen 443;

    server_name example.com;
    location / {
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection upgrade;
        proxy_redirect                      off;
        proxy_set_header Host               $host;
        proxy_set_header X-Real-IP          $remote_addr;
        proxy_set_header X-Forwarded-For    $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto  $scheme;
      proxy_pass      http://127.0.0.1:8080;
       }
    location /socket {
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
        rewrite  ^/socket/(.*) /$1 break;
        proxy_connect_timeout 175s;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
        proxy_pass http://127.0.0.1:8081;
    }
}
```

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
axelar:
  name: axelar
  logo: https://assets.coingecko.com/coins/images/24489/large/tsYr25vB_400x400.jpg
  rpc_url: https://rpc.cosmos.directory/axelar
  rest_url: https://axelar-api.polkachu.com
  wss_url: wss://axelar-rpc.chainode.tech/websocket
```




## Support a new endpoint.

Go to [`src/routes`](https://github.com/testnetrunn/explorer-backend/tree/main/src/routes), and find or create a new file for the category associated with the new endpoint to be supported.

> Don't forget to import important stuff.

Create a new function representing the endpoint like below:
```rs
#[get("{chain}/example")]
pub async fn example(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_example().await?;,
    Ok(TNRAppSuccessResponse::new(data))
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

# Fetching proto 
buf export buf.build/cosmos/cosmos-sdk --output proto
buf export buf.build/cosmos/ibc --output proto
buf export buf.build/evmos/evmos --output proto
buf export buf.build/osmosis-labs/osmosis --output proto
buf export buf.build/umee-network/umee --output proto (complicated not published to buf registry)
buf export buf.build/Gravity-Bridge/Gravity-Bridge --output proto (complicated not publish and non conforming directory structure)

lastly export cosmos-sdk to override purposes