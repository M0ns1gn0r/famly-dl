# What it does

* Loads a list of all children and allows to pick one
* Downloads all Famly posts that have at least one photo tagged with that child
* Creates a folder structure with `index.html` containing links to every downloaded post, and a separate folder with all tagged photos

# Usage

Use your browser's DevTools to get Famly's access token. Set `FAMLY_ACCESS_TOKEN` environment variable:
```ps
# Powershell example.
$env:FAMLY_ACCESS_TOKEN = "00000000-0000-0000-0000-000000000000"
```

Compile and run the program.