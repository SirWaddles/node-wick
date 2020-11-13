# node-wick

NodeJS Bindings for the JohnWickParse library

## Extracting

The `Extractor` class is used to extract files from a .utoc/ucas.

The AES key is ignored in the case of non-encrypted .utoc/ucas files, so an empty string is acceptable in these cases, although null or undefined may produce an error.

### Requirements

Extracting from utoc/ucas archives requires that `oo2core_8_win64.dll` is in the working directory on Windows, and `oo2core_8_win64.so` is present on Linux.

[Linoodle](https://github.com/McSimp/linoodle) is an acceptable substitute for `oo2core_8_win64.so` on Linux systems.

```javascript
const fs = require('fs');
const { Extractor } = require('node-wick');

// Make a new Extractor by specifying the file path (minus the extension), and the AES encryption key as a hexadecimal string.
let extractor = new Extractor("pakchunk", "");

// Iterate over all the files in the ucas, and extract them.
// get_file_list returns an array of file paths within the ucas. You will need the index in the array to extract the files.
extractor.get_file_list().forEach((v, idx) => {
    // get_file(path) returns a NodeJS Buffer with the decrypted file contents.
    fs.writeFileSync(idx + ".uasset", extractor.get_file(v));
});
```

## Parsing

### New Asset Format Requirements

As of Fortnite 14.40, asset files now use a format known as 'Unversioned Properties'. This format is no longer self-describing, and parsing them is significantly more complex. This library attempts to simplify this using 'mapping' files, which contain the description of each class that is necessary to parse that class of asset file.

The official mapping files for `node-wick` are hosted on the [JohnWickParse repository](https://github.com/SirWaddles/JohnWickParse/tree/master/mappings). However, these are incomplete, and are at best, the bare minimum required to parse the item descriptions for the Fortnite shop.

[Lucas7Yoshi](https://twitter.com/Lucas7yoshi) also maintains a repository of asset class mappings, which you can find on his [GitHub repository](https://github.com/Lucas7yoshi/FNMappingsJWP).

[FunGames](https://twitter.com/FunGamesLeaks) and [Officer](https://twitter.com/Not0fficer) also maintain a [repository of mapping files](https://github.com/FabianFG/FortniteTypeMappings), however these are in a different format to that used by `node-wick`, but a converter can be used to translate these mappings.

The mapping files **must** be present in the `mappings` inside the working directory. Any parsing **will fail** if they are not present. There are two types of mapping file, contained in the `mappings/classes` directory and the `mappings/enums` directory.

Parsing assets also requires that the `global.utoc` and `global.ucas` files are present in the `paks/` directory, again in the working directory. You can use `wick-downloader` to download these files if necessary. The global files must match the version of Fortnite that the assets were obtained from.

### Parsing Files

The `Package` class lets you parse files uasset files. It has two constructors:
```javascript
const { Package } = require('node-wick');

// Construct a package from a filepath
let asset = new Package("./assets/test");

// Construct a package from several buffers
// Note that the ubulk buffer is optional, but the parser will error if you try to parse an asset that requires it.
let uassset_buf = fs.readFileSync("./assets/test.uasset");
let ubulk_buf = fs.readFileSync("./assets/test.ubulk");
let asset = new Package(uasset_buf, ubulk_buf);
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
