# Remote file monitor client

This software montor files on remote machines via ssh connection.

## Usage

Provide path to the config file as the first and only argument.

## Configuration

Config file JSON schematic:

```
{
  "type": "object",
  "required": [],
  "properties": {
    "remotes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [],
        "properties": {
          "usr": {
            "type": "string"
          },
          "addr": {
            "type": "string"
          },
          "method": {
            "type": "string"
          },
          "file_path": {
            "type": "string"
          }
        }
      }
    }
  }
}
```
