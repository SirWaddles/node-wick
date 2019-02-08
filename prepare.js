const fs = require('fs');
const neon_build = require('neon-cli/lib/ops/neon_build');

// https://stackoverflow.com/a/32197381/3479580
function deleteFolderRecursive(path) {
    if (fs.existsSync(path)) {
        fs.readdirSync(path).forEach(function(file, index) {
            var curPath = path + "/" + file;
            if (fs.lstatSync(curPath).isDirectory()) { // recurse
                deleteFolderRecursive(curPath);
            } else { // delete file
                fs.unlinkSync(curPath);
            }
        });
        fs.rmdirSync(path);
    }
};

deleteFolderRecursive('./bin-package');
fs.mkdirSync('./bin-package');
neon_build.default(process.cwd());
fs.copyFileSync('./native/index.node', './bin-package/index.node');
