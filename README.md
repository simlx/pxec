# pxec
Execute **scripts** through aliases

# Configuration
Configurable in /config/config:
* editor

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

# pxc architecture
.pxc/<br/>
├── cmd<br/>
│   ├── 0EE20629<br/>
│   ├── 103A40A7<br/>
│   ├── F7265AAD<br/>
│   ├── F8D925B0<br/>
│   └── FFED6378<br/>
├── config<br/>
│   └── config<br/>
├── files<br/>
│   ├── file1<br/>
│   ├── file2<br/>
│   └── file3<br/>
└── map<br/>
└── pxc<br/>

# License
GPLv3
