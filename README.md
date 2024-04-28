# pxec
Execute **scripts** through aliases

# pxc architecture
.pxc/
├── cmd
│   ├── 0EE20629
│   ├── 103A40A7
    ├── EEEF804D
│   ├── F7265AAD
│   ├── F8D925B0
│   └── FFED6378
├── files
└── map
    └── pxc

# Building
``cargo build --release``

# Supported Platforms
``Linux``

# Usage
| input                   | description      |
|-------------------------|------------------|
|                         | show help        |
| ls [category]           | list entries     |
| lsc                     | list categories  | 
| edit [name] [category]  | edit entry       | 
| add [name]              | add entry        | 
| rm [name]               | remove entry     | 
| ext [name]              | export as .pxc   |

# Info
Vim is used as the editor

# License
GPLv3
