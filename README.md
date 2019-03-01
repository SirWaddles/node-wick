# node-wick

NodeJS Bindings for the JohnWickParse library

## Extracting

The `PakExtractor` class is used to extract files from a .pak, and it's use is pretty basic.

```javascript
const fs = require('fs');
const { PakExtractor } = require('node-wick');

// Make a new PakExtractor by specifying the file path, and the AES encryption key as a hexadecimal string.
let extractor = new PakExtractor("pakchunk.pak", "0000000000000000000000000000000000000000000000000000000000000000");

// Iterate over all the files in the pak, and extract them.
// get_file_list returns an array of file paths within the pak. You will need the index in the array to extract the files.
extractor.get_file_list().forEach((v, idx) => {
    // get_file(index) returns a NodeJS Buffer with the decrypted file contents.
    fs.writeFileSync(v, extractor.get_file(idx));
});
```

### Pak Key GUID

The `read_pak_key(path: string)` will return the `EncryptionKeyGuid` part of the header inside the pak file. You do not need the key to get this data.

## Parsing Files

The `read_asset(path: string)` function is the main way to parse an asset file.

The path parameter **should not** contain a file extension - it will load up both the uexp and uasset files.

This function returns a JavaScript object representation of the underlying UObject data. It will be an array of exports, each with the `export_type` parameter.

## Textures

There are two functions to read textures:
 * `read_texture(path: string)`
 * `read_texture_to_file(path: string, output_path: string)`

Both of these functions accept a path of the same format as `read_asset`, and will throw an exception if the `export_type` of the first export is not a `Texture2D`

While `read_texture` returns a NodeJS Buffer containing png data, `read_texture_to_file` will write that content directly to the `output_path` provided.
