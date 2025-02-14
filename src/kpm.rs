/**
 * Kernelino Package Manager
 * 
 */

use std::collections::HashMap;

pub struct KPM {
    pub packages: HashMap<String, Package>,
}

pub struct Package {
    name: String,
    version: String,
    description: String,
    url: String,
    dependencies: Vec<String>,
}