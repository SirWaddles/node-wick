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

## Parsing

The `Package` class lets you parse files uasset/uexp files. It has two constructors:
```javascript
const { Package } = require('node-wick');

// Construct a package from a filepath
let asset = new Package("./assets/test");

// Construct a package from several buffers
// Note that the ubulk buffer is optional, but the parser will error if you try to parse an asset that requires it.
let uassset_buf = fs.readFileSync("./assets/test.uasset");
let uexp_buf = fs.readFileSync("./assets/test.uexp");
let ubulk_buf = fs.readFileSync("./assets/test.ubulk");
let asset = new Package(uasset_buf, uexp_buf, ubulk_buf);
```

You can get a JSON object of the internal parameters of the objects, using the `get_data` method:

```javascript
let asset = new Package("./assets/test");
console.log(JSON.stringify(asset.get_data()));
```

You can also get a buffer of the Texture contained within the asset (if there is one) using the `get_texture` method:

```javascript
let asset = new Package("./assets/test");
fs.writeFileSync("./assets/test.png", asset.get_texture());
```

Note that retrieving the texture from a Package asset, will invalidate the package (it can no longer be used.)

### Pak Key GUID

The `read_pak_key(path: string)` will return the `EncryptionKeyGuid` part of the header inside the pak file. You do not need the key to get this data.
