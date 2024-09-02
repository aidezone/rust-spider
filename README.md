# rust-spider
rust实现的抓站系统

# 目录结构
```
rust-spider/
│
├── src/
│   ├── main.rs                 # 入口文件，解析命令行参数，启动爬虫
│   ├── worker/                 # 工作模块，负责任务调度和执行
│   │   ├── mod.rs              # worker模块入口
│   │   ├── task.rs             # 任务记录与管理
│   │   └── worker.rs           # 线程工作逻辑实现
│   ├── fetcher/                # 抓取模块
│   │   ├── mod.rs              # 抓取模块入口
│   │   └── fetch.rs            # URL抓取实现
│   ├── parser/                 # 解析模块
│   │   ├── mod.rs              # 解析模块入口
│   │   └── html_parser.rs      # HTML解析实现
│   ├── writer/                 # 写入模块
│   │   ├── mod.rs              # 写入模块入口
│   │   └── markdown_writer.rs  # Markdown写入实现
│   └── utils/                  # 工具模块
│       ├── mod.rs              # 工具函数与辅助代码
│       ├── cli.rs              # 命令行参数解析实现
│       └── logging.rs          # 日志打印相关方法实现
│
├── Cargo.toml                  # Cargo配置文件
└── visited_urls.txt            # 任务状态文件，记录已访问URL
```
# 编译
```bash
cargo build --release
```

# 命令运行帮助
```
spider 1.0
Gao Yuan <yuan.gao@13un.com>
A simple web crawler

USAGE:
    spider [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    create     Creates a new crawling task
    help       Print this message or the help of the given subcommand(s)
    list       Lists all crawling tasks
    restart    Restarts a crawling task
    resume     Resumes a crawling task
```

# Example
```bash
# 创建一个任务（ctrl+c可中断并退出运行）运行过程中输出总体进度以及正在抓取的站点
spider create -f url_list.txt -n first_task_by_gaoyuan -d 3

# 任务记录列表，打印一个命令行表格，显示任务id、任务名称、进度、创建时间、结束时间
spider list

# 恢复一个任务
spider resume -id 1

# 重置任务进度并重启
spider restart -id 1
```
