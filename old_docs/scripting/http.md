# HTTP

If you want to make a http request you have to import the `nog/http` module.

The module provides you with following functions:

* post
* get
* patch
* put
* delete

Each function has the following signature:

```
import "nog/http" as HTTP;

let response = HTTP::get(<url> [, <body>]);
```

**Note**: The http module is synchronous, so using this module can slow down/freeze the application.

The response has the following properties:

| Key          | Value  | Description               |
|--------------|--------|---------------------------|
| body         | Object | The response body         |
| content_type | String | The response content type |
| status_code  | Number | The response status       |

## Usage

**GET**
```nog
import "nog/http" as HTTP;

const response = HTTP::get("https://www.google.com");

print(response.body); //prints the html of google
```