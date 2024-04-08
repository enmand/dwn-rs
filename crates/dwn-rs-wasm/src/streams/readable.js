// Provide a JavaScript shim for the WAS-based ReadableStream, so we can use it in
// the browser and we can use the ES6-style classes

const { Readable } = require("readable-stream");

const makeReadable = (readFn, abort) => {
  return new Readable({
    read(size) {
      this.push(readFn(size));
    },

    signal: abort,
  });
};

module.exports = { makeReadable };
