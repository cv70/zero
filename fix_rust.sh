#!/bin/bash
find zero-core/src -name "*.rs" -exec sed -i s/#\[derive(Debug, Clone\]/#[derive(Debug, Clone)]/g {} \\\;
echo Fixed all Rust files
