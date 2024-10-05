#!/usr/bin/env node

const { run } = require('./index');

// Remove the first two arguments (node executable and script path)
const args = process.argv.slice(2);

console.log("Forwarding arguments to Rust:", args);

try {
    run(args);
} catch (error) {
    console.error("Error running create-v1-app:", error);
}