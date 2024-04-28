# pxec
Execute **scripts** through aliases

# pxc architecture
.pxc/<br/>
├── cmd<br/>
│   ├── 0EE20629<br/>
│   ├── 103A40A7<br/>
│   ├── EEEF804D<br/>
│   ├── F7265AAD<br/>
│   ├── F8D925B0<br/>
│   └── FFED6378<br/>
├── files<br/>
└── map<br/>
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
