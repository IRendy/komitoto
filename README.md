# komitoto

HAM Radio QSO Logbook Manager - 业余无线电通联日志管理工具

## 快速开始

```bash
# 初始化配置
komitoto config init

# 查看当前日志本列表
komitoto logbook list

# 添加一条通联记录
komitoto log add --call BI3SEY --freq 438.500 --mode FT8 --date 20251215

# 列出所有通联
komitoto log list

# 查看日出时间
komitoto calc sunrise --lat 39.9042 --lon 116.4074
```

---

## 命令使用指南

### `komitoto config` — 配置管理

#### `komitoto config init`

初始化配置文件 `komitoto.toml`，从示例模板创建或生成默认配置。

```bash
komitoto config init
```

示例：

```bash
# 在当前目录创建配置
komitoto config init
Created komitoto.toml from example file

Please edit komitoto.toml and set your information.
```

#### `komitoto config show`

显示当前配置文件的所有内容。

```bash
komitoto config show
```

输出示例：

```
Current configuration (komitoto.toml):
==================================================
# Your callsign (required for ADIF export)
callsign = "BD7ACE"

# Your real name or nickname
name = "Your Name"

# QTH (QRA locator description, e.g., city/district)
qth = "Beijing, China"

# Country (full name or code)
country = "China"

# Equipment - Transmitter/Rig
rig = "IC-7300"

# Equipment - Receiver
rx = "IC-7300"

# Your grid square (Maidenhead locator)
grid = "OL82tk"

# Timezone (IANA timezone format)
timezone = "Asia/Shanghai"

# Your altitude above sea level (meters)
my_altitude = 120

# Transmit power (in watts)
tx_power = 100

# Default RST values to speed up logging
rst_sent_default = "59"
rst_rcvd_default = "59"
```

#### `komitoto config set`

更新配置文件的特定字段。

```bash
komitoto config set <FIELD> <VALUE>
```

可用字段：

| 字段 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `callsign` | String | 您的呼号 | `"BG3WHP"` |
| `name` | String | 姓名/昵称 | `"张三"` |
| `qth` | String | QTH 位置 | `"Beijing, China"` |
| `country` | String | 国家 | `"China"` |
| `rig` | String | 发射电台 | `"IC-7300"` |
| `rx` | String | 接收机 | `"IC-7300"` |
| `grid` | String | 网格定位 | `"OL82tk"` |
| `timezone` | String | 时区（IANA 格式） | `"America/New_York"` |
| `my_altitude` | Integer | 海拔高度（米） | `120` |
| `my_antenna` | String | 天线类型 | `"Dipole 40m/15m"` |
| `my_city` | String | 城市 | `"Beijing"` |
| `tx_power` | Integer | 发射功率（瓦） | `100` |
| `rx_power` | Integer | 接收功率 | `0` |
| `rst_sent_default` | String | RST 发送默认值 | `"59"` |
| `rst_rcvd_default` | String | RST 接收默认值 | `"59"` |

示例：

```bash
# 设置呼号
komitoto config set callsign "BG3WHP"
Updated callsign: BG3WHP

# 设置发射功率
komitoto config set tx_power 50
Updated tx_power: 50

# 设置海拔
komitoto config set my_altitude 120
Updated my_altitude: 120

# 更改时区
komitoto config set timezone "America/New_York"
Updated timezone: America/New_York

# 设置网格坐标
komitoto config set grid "OL82tk"
Updated grid: OL82tk

# 设置默认 RST 值
komitoto config set rst_sent_default "59"
komitoto config set rst_rcvd_default "59"

# 错误处理：无效字段
komitoto config set tz invalid
Unknown field: tz. Available fields:
  callsign, name, qth, country, rig, rx, grid, timezone
  my_altitude, my_antenna, my_city, tx_power, rx_power
  rst_sent_default, rst_rcvd_default
```

时区格式参考：

- `Asia/Shanghai` - 北京时间
- `America/New_York` - 纽约时间
- `Europe/London` - 伦敦时间
- `Australia/Sydney` - 悉尼时间

---

### `komitoto logbook` — 日志本管理

#### `komitoto logbook list`

列出当前目录中所有可用的日志本文件。

```bash
komitoto logbook list
```

输出示例：

```
Available logbooks in current directory:
  - ./export_test.json
  - ./logbook#1.adi
  - ./logbook#1_corrected.adi
  - ./qsos.csv
  - ./test.csv
  - ./test.json
  - test_export.adi
  - komitoto.db (default)
```

#### `komitoto logbook use`

切换到指定的日志本文件。支持 `.adi`, `.json`, `.db` 等格式。

```bash
komitoto logbook use <FILE>
```

示例：

```bash
# 切换到 ADI 日志本
komitoto logbook use logbook#1.adi
Importing logbook#1.adi to temporary database...
Created temporary database: ./logbook#1_temp.db with 5 QSOs

# 切换回主数据库
komitoto logbook use komitoto.db
Using logbook: komitoto.db

# 错误处理：不存在文件
komitoto logbook use nonexistent.xyz
Importing nonexistent_file.xyz to temporary database...
Error: expected value at line 1 column 1
```

---

### `komitoto log` — 通联日志管理

#### `komitoto log add`

添加新的 QSO 记录。

```bash
komitoto log add [OPTIONS]
```

选项：

| 选项 | 简写 | 说明 |
|------|------|------|
| `--call` | `-c` | 对方呼号 **必填** |
| `--freq` | `-f` | 频率 (MHz) **必填** |
| `--mode` | `-m` | 模式 (CW, FM, SSB, FT8, RTTY, AM, SSTV 等) **必填** |
| `--date` | | 日期，格式 YYYYMMDD，默认今天 |
| `--time` | | 时间，格式 HHMMSS，默认当前 UTC |
| `--now` | | 使用当前 UTC 时间（默认行为） |
| `--rst-sent` | | 发送 RST |
| `--rst-rcvd` | | 接收 RST |
| `--grid` | | Grid Locator（网格坐标） |
| `--qth` | | QTH（位置） |
| `--rig` | | 使用电台 |
| `--name` | | 操作员姓名 |
| `--comment` | | 备注 |
| `--json` | | 从 JSON 字符串添加 QSO |

示例：

```bash
# 快速记录当前 QSO
komitoto log add --call BI3SEY --now --freq 438.500 --mode FT8 --rst-sent 59 --rst-rcvd 59
QSO added: BI3SEY (uuid...) @ 438.500 MHz FT8 2026-04-27 16:42:00 UTC

# 指定日期时间
komitoto log add --call BA7LU --freq 7.020 --mode cw --date 20251225 --time 140000 --rst-sent 599 --rst-rcvd 599
QSO added: BA7LU (uuid...) @ 7.020 MHz CW 2025-12-25 14:00:00 UTC

# 带完整信息记录
komitoto log add --call BG5ABC --freq 14.100 --mode SSB --grid OL72wx --qth Chengdu --rig IC-7610 --name "Test User" --comment "First QSO in Chengdu"

# 从 JSON 添加
komitoto log add --json '[{"id":"test-uuid","call":"BD8XYZ","freq":14.225,"mode":"FT8","date_time_on":"2026-05-01T10:30:00Z","grid":"OL72yy"}]'
QSO added: BD8XYZ (test-uuid) @ 14.225 MHz FT8 2026-05-01 10:30:00 UTC

# 错误处理：无效模式
komitoto log add --call BD9XYZ --freq 14.0 --mode INVALID_MODE
Error: Unknown mode: INVALID_MODE

# 错误处理：负频率
komitoto log add --call BD9XYZ --freq -14.0 --mode SSB
error: unexpected argument '-1' found
```

#### `komitoto log list`

列出 QSO 记录，按时间倒序排列。

```bash
komitoto log list [OPTIONS]
```

选项：

| 选项 | 简写 | 说明 |
|------|------|------|
| `--limit` | `-l` | 显示条数，默认 20 |
| `--json` | `-j` | 以 JSON 格式输出 |

示例：

```bash
# 列出最新 20 条
komitoto log list
ID                                   Call             Freq Mode   S     R     Grid     Time (UTC)
----------------------------------------------------------------------------------------------------
test-uui                             BD8XYZ         14.225 FT8    -     -     OL72yy   2026-05-01 10:30
a31a5ac8                             BD7ABC         14.050 RTTY   -     -     -        2026-04-27 16:42
26004497                             BD9XYZ         14.000 SSB    -     -     -        2026-04-27 16:42
1956dc1d                             BA5JKL         21.100 FT8    -     -     50       2026-04-25 12:00

# 限制显示条数
komitoto log list --limit 10

# JSON 输出
komitoto log list --json > today_qsos.json
```

#### `komitoto log get`

根据 ID 查看单条 QSO 详情。**支持部分 ID 匹配（前 6-8 位字符）**。

```bash
komitoto log get [OPTIONS] <ID>
```

选项：

| 选项 | 简写 | 说明 |
|------|------|------|
| `--json` | `-j` | 以 JSON 格式输出 |

示例：

```bash
# 使用完整 UUID
komitoto log get c3cc833d-d980-4f21-9d14-8e341bd64c1f
QSO Detail
==================================================
  ID:       c3cc833d-d980-4f21-9d14-8e341bd64c1f
  Call:     BA5JKL
  Freq:     21.100 MHz
  Band:     15m
  Mode:     FT8
  Date/On:  2026-04-25 12:00:00 UTC
  RST S/R:  50/50
  Grid:     PL56xy

# 使用完整 ID 的 JSON 输出
komitoto log get c3cc833d-d980-4f21-9d14-8e341bd64c1f --json
[{"id":"c3cc833d-d980-4f21-9d14-8e341bd64c1f", ...}]

# 使用前缀 ID（推荐）- 只需输入前 6-8 个字符
komitoto log get c3cc833d
QSO Detail
==================================================
  ID:       c3cc833d-d980-4f21-9d14-8e341bd64c1f
  Call:     BA5JKL
  ...

# 错误处理：不存在的 ID
komitoto log get nonexistent_id
Error: QSO not found with ID 'nonexistent_id' or prefix 'nonexistent_id'

# 错误处理：模糊匹配（多个结果）
komitoto log get ambiguous_prefix
# 会返回找到多个匹配的提示
```

#### `komitoto log update`

更新指定 QSO 记录的字段。**支持部分 ID 匹配**。

```bash
komitoto log update [OPTIONS] <ID>
```

选项：

| 选项 | 简写 | 说明 |
|------|------|------|
| `--call` | `-c` | 对方呼号 |
| `--freq` | `-f` | 频率 (MHz) |
| `--mode` | `-m` | 模式 |
| `--rst-sent` | | 发送 RST |
| `--rst-rcvd` | | 接收 RST |
| `--grid` | | Grid Locator |
| `--qth` | | QTH |
| `--rig` | | 使用电台 |
| `--name` | | 操作员姓名 |
| `--comment` | | 备注 |

示例：

```bash
# 使用完整 UUID 更新
komitoto log update c3cc833d-d980-4f21-9d14-8e341bd64c1f --rst-sent 559
QSO updated: c3cc833d-d980-4f21-9d14-8e341bd64c1f

# 使用前缀 ID 更新（推荐）
komitoto log update c3cc833d --freq 14.225 --comment "Corrected frequency"
QSO updated: c3cc833d-d980-4f21-9d14-8e341bd64c1f

# 同时更新多个字段
komitoto log update bd269759 --freq 14.225 --comment "Test update"
QSO updated: bd269759-840f-4c99-86e0-f752354fb3f1

# 只更新单个字段
komitoto log update <ID> --rst-sent 559
komitoto log update <ID> --grid OM44lg --comment "QSL confirmed"
```

#### `komitoto log delete`

删除指定 QSO 记录。**支持部分 ID 匹配**。

```bash
komitoto log delete <ID>
```

示例：

```bash
# 使用完整 UUID 删除
komitoto log delete c3cc833d-d980-4f21-9d14-8e341bd64c1f
QSO deleted: c3cc833d-d980-4f21-9d14-8e341bd64c1f

# 使用前缀 ID 删除（推荐）
komitoto log delete c3cc833d
QSO deleted (matched by prefix): c3cc833d -> c3cc833d-d980-4f21-9d14-8e341bd64c1f

# 删除后验证
komitoto log list | grep BA5JKL
# 该 QSO 已不在列表中
```

#### `komitoto log import`

从文件导入 QSO 记录，支持多种格式。**自动检测格式**或通过 `--format` 参数指定。

```bash
komitoto log import [OPTIONS] <FILE>
```

选项：

| 选项 | 简写 | 说明 |
|------|------|------|
| `--format` | `-f` | 文件格式：adi, adx, csv, json, sqlite3 |

支持的格式及扩展名自动检测：

| 格式 | 扩展名 |
|------|--------|
| ADIF | `.adi`, `.adif` |
| ADX | `.adx` |
| CSV | `.csv` |
| JSON | `.json` |
| SQLite3 | `.db`, `.sqlite`, `.sqlite3` |

示例：

```bash
# 自动检测格式导入
komitoto log import qsos.csv
Imported 16 QSO(s) from qsos.csv (format: csv)

komitoto log import log.adi
Imported 5 QSO(s) from log.adi (format: adi)

# 指定格式导入
komitoto log import data.json --format json
komitoto log import backup.db --format sqlite3
komitoto log import export.adx --format adx

# 错误处理：文件不存在
komitoto log import nonexistent.json
Error: No such file or directory (os error 2)

# 错误处理：重复 ID
komitoto log import existing_data.json
Error: UNIQUE constraint failed: qsos.id
```

#### `komitoto log export`

导出全部 QSO 记录到文件，支持多种格式。

```bash
komitoto log export [OPTIONS] <FILE>
```

选项：

| 选项 | 简写 | 说明 |
|------|------|------|
| `--format` | `-f` | 导出格式：adi, adx, csv, json, sqlite3 |

示例：

```bash
# 导出为 ADIF 格式
komitoto log export log.adi
Exported 25 QSO(s) to log.adi (format: adi)

# 导出为 JSON 格式
komitoto log export backup.json --format json
Exported 25 QSO(s) to backup.json (format: json)

# 导出为 CSV 格式
komitoto log export qsos.csv --format csv
Exported 25 QSO(s) to qsos.csv (format: csv)

# 导出为 SQLite3 格式
komitoto log export archive.db --format sqlite3
Exported 25 QSO(s) to archive.db (format: sqlite3)

# 导出为 ADX 格式
komitoto log export data.adx --format adx
Exported 25 QSO(s) to data.adx (format: adx)

# 通过文件扩展名指定格式
komitoto log export output.xml --format adx
ADIF_VER=3_1_7
<PROGRAMID:8>komitoto<ADIF_VER:5>3.1.7<eoh>
<QSO_DATE:8>20260501<TIME_ON:6>103000<CALL:6>BD8XYZ<FREQ:9>14.225000<MODE:3>FT8<GRID:6>OL72yy<eor>
```

---

### `komitoto calc` — 计算工具

#### `komitoto calc sunrise`

计算指定位置的日出、日落及晨光/暮光时间。

```bash
komitoto calc sunrise [OPTIONS] --lat <LAT> --lon <LON>
```

选项：

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--lat` | 纬度（十进制度数，正值为北纬） **必填** | - |
| `--lon` | 经度（十进制度数，正值为东经） **必填** | - |
| `--date` | 日期，格式 YYYYMMDD | 今天 |
| `--altitude` | 海拔（米） | 0 |
| `--dawn` | 晨光类型：civil, nautical, astronomical | civil |
| `--json` | 以 JSON 格式输出 | 否 |

晨光类型说明：

| 类型 | 说明 | 太阳位于地平线下的角度 |
|------|------|--------------------------|
| `civil` | 民用晨光/暮光 | 0° ~ 6° |
| `nautical` | 航海晨光/暮光 | 6° ~ 12° |
| `astronomical` | 天文晨光/暮光 | 12° ~ 18° |

输出同时显示 UTC 和北京时间（BJT, UTC+8）。

示例：

```bash
# 计算广州今日日出日落
komitoto calc sunrise --lat 23.1291 --lon 113.2644
Sun times for 20260427 at (23.1291, 113.2644) alt=0m
==================================================
  Sunrise:  21:59:53 UTC (BJT: 05:59:53)
  Sunset:   10:50:27 UTC (BJT: 18:50:27)
  Dawn:     21:36:36 UTC (BJT: 05:36:36)
  Dusk:     11:13:44 UTC (BJT: 19:13:44)

# 计算北京指定日期
komitoto calc sunrise --lat 39.9042 --lon 116.4074 --date 20250115
Sun times for 20250115 at (39.9042, 116.4074) alt=0m
==================================================
  Sunrise:  23:34:04 UTC (BJT: 2025-01-15 07:34:04)
  Sunset:   09:12:53 UTC (BJT: 2025-01-15 17:12:53)
  Dawn:     23:04:21 UTC (BJT: 2025-01-15 07:04:21)
  Dusk:     09:42:36 UTC (BJT: 2025-01-15 17:42:36)

# 使用航海晨光
komitoto calc sunrise --lat 39.9042 --lon 116.4074 --dawn nautical

# 指定海拔
komitoto calc sunrise --lat 23.1291 --lon 113.2644 --altitude 100

# 不同半球测试 - 纽约
komitoto calc sunrise --lat 40.7128 --lon=-74.0060 --date 20250621 --altitude 10
Sun times for 20250621 at (40.7128, -74.006) alt=10m
==================================================
  Sunrise:  09:24:20 UTC (BJT: 2025-06-21 17:24:20)
  Sunset:   00:31:20 UTC (BJT: 2025-06-22 08:31:20)

# 南半球 - 布宜诺斯艾利斯
komitoto calc sunrise --lat=-35 --lon=-58 --date 20250621 --altitude=-10
Sun times for 20250621 at (-35, -58) alt=-10m
==================================================
  Sunrise:  11:00:28 UTC (BJT: 2025-06-21 19:00:28)

# JSON 输出
komitoto calc sunrise --lat 39.9042 --lon 116.4074 --json
{
  "date": "20250115",
  "lat": 39.9042,
  "lon": 116.4074,
  "altitude": 0,
  "sunrise": "2025-01-14T23:34:04+00:00",
  "sunset": "2025-01-15T09:12:53+00:00",
  "dawn": "2025-01-14T23:04:21+00:00",
  "dusk": "2025-01-15T09:42:36+00:00"
}

# 错误处理：极地特殊情况
komitoto calc sunrise --lat 90 --lon 0 --date 20250621
thread 'main' panicked at src/calc.rs:23:64:
called `Option::unwrap()` on a `None` value
```

#### `komitoto calc distance`

计算两点之间的距离。

```bash
komitoto calc distance [OPTIONS] --from-lat <FROM_LAT> --from-lon <FROM_LON> --to-lat <TO_LAT> --to-lon <TO_LON>
```

选项：

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--from-lat` | 起点纬度（十进制度数） | - |
| `--from-lon` | 起点经度（十进制度数） | - |
| `--to-lat` | 终点纬度（十进制度数） | - |
| `--to-lon` | 终点经度（十进制度数） | - |
| `--unit` | 单位：km 或 miles | km |
| `--json` | 以 JSON 格式输出 | 否 |

示例：

```bash
# 计算北京到上海的距离
komitoto calc distance --from-lat 39.9042 --from-lon 116.4074 --to-lat 30.2428 --to-lon 121.4737
Distance from (39.9042, 116.4074) to (30.2428, 121.4737): 1166.66 km

# 计算北京到广州的距离
komitoto calc distance --from-lat 39.9042 --from-lon 116.4074 --to-lat 23.1291 --to-lon 113.2644
Distance from (39.9042, 116.4074) to (23.1291, 113.2644): 1883.50 km

# 英里单位
komitoto calc distance --from-lat 39.9042 --from-lon 116.4074 --to-lat 30.2428 --to-lon 121.4737 --unit miles
Distance from (39.9042, 116.4074) to (30.2428, 121.4737): 724.93 miles

# JSON 输出
komitoto calc distance --from-lat 39.9042 --from-lon 116.4074 --to-lat 30.2428 --to-lon 121.4737 --json
{
  "from": { "lat": 39.9042, "lon": 116.4074 },
  "to": { "lat": 30.2428, "lon": 121.4737 },
  "distance_km": 1166.66
}
```

#### `komitoto calc zone`

计算指定坐标的 CQ 分区和 ITU 分区。

```bash
komitoto calc zone [OPTIONS] --lat <LAT> --lon <LON>
```

选项：

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--lat` | 纬度（十进制度数，正值为北纬）**必填** | - |
| `--lon` | 经度（十进制度数，正值为东经）**必填** | - |
| `--json` | 以 JSON 格式输出 | 否 |

示例：

```bash
# 查询北京的分区
komitoto calc zone --lat 39.9042 --lon 116.4074
Location: (39.9042, 116.4074)
CQ Zone: 24
ITU Zone: 44

# 查询香港的分区
komitoto calc zone --lat 22.3193 --lon 114.1694
Location: (22.3193, 114.1694)
CQ Zone: 24
ITU Zone: 44

# JSON 输出
komitoto calc zone --lat 39.9042 --lon 116.4074 --json
{
  "location": { "lat": 39.9042, "lon": 116.4074 },
  "cq_zone": { "type": "CQ", "number": 24 },
  "itu_zone": { "type": "ITU", "number": 44 }
}
```

---

#### `komitoto calc coordinate`

在 Maidenhead 网格定位和经纬度之间互相转换。

```bash
komitoto calc coordinate [OPTIONS]
```

选项：

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--from` | 输入格式：`grid` 或 `latlon`（可自动推断） | 自动推断 |
| `--to` | 输出格式：`grid` 或 `latlon`（可自动推断） | 自动推断 |
| `--lat` | 纬度（十进制度数，正值为北纬） | - |
| `--lon` | 经度（十进制度数，正值为东经） | - |
| `--input` | 输入值（网格如 `OL82tk` 或坐标如 `39.9042,116.4074`） | - |
| `--precision` | 网格精度：2, 4, 6, 8 或 10 | 6 |
| `--json` | 以 JSON 格式输出 | 否 |

支持多种输入方式：
- 经纬度转网格：`--lat/--lon` 或 `--input "lat,lon"`
- 网格转经纬度：`--input GRID`

示例：

```bash
# 北京经纬转网格（默认 6 字符精度）
komitoto calc coordinate --lat 39.9042 --lon 116.4074
Input: (39.9042, 116.4074)
Grid: OM89ev

# 使用 input 参数
komitoto calc coordinate --input "39.9042,116.4074"
Input: (39.9042, 116.4074)
Grid: OM89ev

# 网格转经纬度
komitoto calc coordinate --input OL82tk
Input: OL82tk
Latitude: 22.4375
Longitude: 117.625

# 不同精度
komitoto calc coordinate --lat 39.9042 --lon 116.4074 --precision 4
Input: (39.9042, 116.4074)
Grid: OM89

komitoto calc coordinate --lat 39.9042 --lon 116.4074 --precision 2
Input: (39.9042, 116.4074)
Grid: OM

# JSON 输出
komitoto calc coordinate --lat 39.9042 --lon 116.4074 --json
{
  "latitude": 39.9042,
  "longitude": 116.4074,
  "grid": "OM89ev"
}

komitoto calc coordinate --input OL82tk --json
{
  "input": "OL82tk",
  "latitude": 22.4375,
  "longitude": 117.625
}

# 显式指定转换方向
komitoto calc coordinate --from grid --to latlon --input OL82tk
komitoto calc coordinate --from latlon --to grid --lat 39.9042 --lon 116.4074
```

#### `komitoto calc sstv` — SSTV 慢扫描电视

将图像编码为 SSTV 音频，或将 SSTV 音频解码为图像。支持 14 种 SSTV 模式。

**命令：**

| 子命令 | 说明 |
|--------|------|
| `encode` | 将图像编码为 SSTV 音频 (WAV) |
| `decode` | 将 SSTV 音频解码为图像 |
| `info`   | 显示指定 SSTV 模式的详细信息 |
| `list`   | 列出所有支持的 SSTV 模式 |

**支持的 SSTV 模式 (14 种)：**

| 模式 | 分辨率 | 时长 | 说明 |
|------|--------|------|------|
| Martin M1 | 320×256 | ~113s | 最常用的 HF SSTV 模式 |
| Martin M2 | 320×256 | ~57s | M1 的半速版本 |
| Scottie S1 | 320×256 | ~112s | 另一种常用模式 |
| Scottie S2 | 320×256 | ~71s | S1 的较短版本 |
| Robot 36 | 320×240 | ~44s | YUV 色彩空间，NTSC 风格 |
| Robot 72 | 320×240 | ~72s | 更高质量的 Robot |
| PD 50/90/120/180/240/290 | 320×256 | 可变 | 双音同步模式 |
| AVT 90/120 | 320×256 | 可变 | VOX 友好的双音模式 |

**示例：**

```bash
# 将图像编码为 SSTV 音频 (默认 Martin M1, fit 缩放)
komitoto calc sstv encode photo.png

# 指定模式、裁剪策略和输出文件
komitoto calc sstv encode photo.png -m scotties1 -s crop -o tx.wav

# 将 SSTV WAV 音频解码为图像
komitoto calc sstv decode received.wav -m martinm1

# 将 SSTV MP3 音频解码为图像 (自动重采样)
komitoto calc sstv decode received.mp3 -m martinm1 -o decoded.png

# 查看模式信息
komitoto calc sstv info pd90
Mode: PD 90
Resolution: 320x256
Sample rate: 11025 Hz
Encoding: ~105 seconds for a full image

# 列出所有模式
komitoto calc sstv list

# 转换为 MP3 (使用 ffmpeg)
komitoto calc sstv encode photo.png -o photo.wav
ffmpeg -i photo.wav -codec:a libmp3lame -qscale:a 2 photo.mp3
```

**完整案例（含输出）：**

```bash
# 1. 列出所有支持的 SSTV 模式
$ komitoto calc sstv list
Supported SSTV modes:
  Martin M1 (320x256)
  Martin M2 (320x256)
  Scottie S1 (320x256)
  Scottie S2 (320x256)
  Robot 36 (320x240)
  Robot 72 (320x240)
  PD 50 (320x256)
  PD 90 (320x256)
  PD 120 (320x256)
  PD 180 (320x256)
  PD 240 (320x256)
  PD 290 (320x256)
  AVT 90 (320x256)
  AVT 120 (320x256)

# 2. 使用 Martin M1 编码图像（最常用的 HF 模式）
$ komitoto calc sstv encode cq.png -m m1 -o cq_m1.wav
SSTV audio generated: cq_m1.wav
  Mode: Martin M1
  Resolution: 320x256
  Image: cq.png
  Resize: fit

# 3. 使用 Robot 36 编码（快速模式，YUV 色彩）
$ komitoto calc sstv encode photo.jpg -m r36 -o fast.wav
SSTV audio generated: fast.wav
  Mode: Robot 36
  Resolution: 320x240
  Image: photo.jpg
  Resize: fit

# 4. 使用 Scottie S1 + crop 策略（裁剪图像适应 320×256）
$ komitoto calc sstv encode wide.png -m s1 -s crop -o scottie.wav
SSTV audio generated: scottie.wav
  Mode: Scottie S1
  Resolution: 320x256
  Image: wide.png
  Resize: crop

# 5. 使用 PD 90 编码（双音同步模式）
$ komitoto calc sstv encode photo.png -m pd90 -o pd90.wav
SSTV audio generated: pd90.wav
  Mode: PD 90
  Resolution: 320x256
  Image: photo.png
  Resize: fit

# 6. 解码 WAV 音频
$ komitoto calc sstv decode cq_m1.wav -m m1 -o decoded.png
SSTV decoded: decoded.png
  Mode: Martin M1
  Resolution: 320x256
  Audio: cq_m1.wav

# 7. 解码 MP3 音频（自动重采样 44100→11025 Hz）
$ komitoto calc sstv decode received.mp3 -m m1 -o from_mp3.png
SSTV decoded: from_mp3.png
  Mode: Martin M1
  Resolution: 320x256
  Audio: received.mp3

# 8. 批量编码不同模式
$ for m in m1 m2 s1 s2 r36; do
    komitoto calc sstv encode test.png -m $m -o test_$m.wav
  done

# 9. 编码后转换为 MP3 传输
$ komitoto calc sstv encode photo.png -m m1 -o photo.wav
$ ffmpeg -i photo.wav -codec:a libmp3lame -qscale:a 2 photo.mp3

# 10. 错误处理示例

# 文件不存在
$ komitoto calc sstv encode missing.png -m m1
Error: Image file not found: missing.png

# 不支持的图像格式
$ komitoto calc sstv encode doc.txt -m m1
Error: Unsupported image format: txt (supported: png, jpg, jpeg, bmp, gif, webp, tiff, tif)

# 不支持的音频格式
$ komitoto calc sstv decode audio.flac -m m1
Error: Unsupported audio format: flac (supported: wav, mp3)

# 无效模式
$ komitoto calc sstv encode photo.png -m badmode
Error: Unknown SSTV mode: 'badmode'. Use --list to see available modes.
```

**常用场景：**

| 场景 | 推荐模式 | 命令 |
|------|----------|------|
| HF 通联（高质量） | Martin M1 | `komitoto calc sstv encode photo.png -m m1` |
| HF 通联（快速） | Martin M2 | `komitoto calc sstv encode photo.png -m m2` |
| VHF/UHF 通联 | Robot 36 | `komitoto calc sstv encode photo.png -m r36` |
| 宽频带、高质量 | PD 90/120 | `komitoto calc sstv encode photo.png -m pd90` |
| VOX 操作 | AVT 90/120 | `komitoto calc sstv encode photo.png -m avt90` |

**encode 选项：**

| 选项 | 简写 | 默认值 | 说明 |
|------|------|--------|------|
| `<image>` | | (必需) | 输入图像文件 (png, jpg, bmp, gif, webp, tiff) |
| `--output` | `-o` | `<image_stem>.wav` | 输出 WAV 文件路径 |
| `--mode` | `-m` | `martinm1` | SSTV 模式 (如 m1, s1, r36, pd90) |
| `--strategy` | `-s` | `fit` | 图像处理策略: crop, fit, stretch |

**decode 选项：**

| 选项 | 简写 | 默认值 | 说明 |
|------|------|--------|------|
| `<audio>` | | (必需) | 输入音频文件 (wav, mp3) |
| `--output` | `-o` | `<audio_stem>.png` | 输出图像文件路径 |
| `--mode` | `-m` | `martinm1` | SSTV 模式 (或 auto) |

---

## 工作流程示例

### 基本工作流程

```bash
# 1. 初始化配置
komitoto config init
# 编辑 komitoto.toml 填入个人信息

# 2. 添加通联记录
komitoto log add --call BI3SEY --freq 438.500 --mode FT8 --date 20251215 --rst-sent 59 --rst-rcvd 59

# 3. 查询记录
komitoto log list
komitoto log get bi3se  # 使用部分 ID 查找

# 4. 修正数据
komitoto log update bi3se --freq 438.600 --comment "Frequency corrected"

# 5. 导出数据
komitoto log export my_log.adi --format adi
komitoto log export backup.json --format json
```

### 多日志本工作流

```bash
# 查看可用日志本
komitoto logbook list

# 临时查看另一个日志本
komitoto logbook use logbook#1.adi
komitoto log list

# 切回主日志本
komitoto logbook use komitoto.db

# 永久合并两个日志本（将外部日志本导入主库）
komitoto log import logbook#1.adi --format adi
```

### 批量导入示例

```bash
# 从其他软件导出的 CSV 导入
komitoto log import ham_tools_export.csv

# 从 ClubLog 导出的 ADIF 导入
komitoto log import clublog_export.adi

# 从 QRZ.com 导出的 JSON 导入
komitoto log import qrz_export.json
```

---

## 旧版命令参考（已废弃/可能不适用）

以下为早期设计文档中的命令，可能与当前实现不符：

```bash
# 旧的版本命令格式
komitoto version

# 已过时的配置命令
komitoto config view
komitoto config set default_rig IC7300

# 已废弃的日志本操作
komitoto logbook create Logbook#001
komitoto logbook info
komitoto logbook copy Logbook#001 Logbook#001_backup
komitoto logbook delete Logbook#001
komitoto logbook current

# 未实现的设备控制功能（占位符）
komitoto rig list
komitoto rig add --name IC7300 --port /dev/ttyUSB0
komitoto rig status
komitoto rig set --freq 7.050M
komitoto rig ptt on
komitoto rig ptt off
komitoto rig cw send "HPE CU AGE VY 73 SK E E"
komitoto rig ft8 monitor
komitoto rig ft8 cq

# 未实现的卫星相关功能
komitoto sat

# 未实现的指向器功能
komitoto rotator

# 未实现的网格计算
komitoto calc grid --lat --lon

# 未实现的频率计算器
komitoto calc freq

# 未实现的多普勒计算器
komitoto calc doppler

# 旧的搜索命令格式（已被部分 ID 匹配取代）
komitoto log search --call "BI3*"
```

---

## 技术栈

- **clap v4.6.1** - CLI 框架
- **rusqlite 0.34** - SQLite 数据库支持
- **serde + serde_json** - JSON 序列化
- **toml 0.8** - TOML 配置文件解析
- **chrono-tz** - 时区处理
- **geo** - 地理距离计算（VincentyDistance）和区域查找
- **sunrise** - 天文计算
- **uuid** - UUID 生成
- **lazy_static** - 全局变量缓存

---

## 许可证

MIT License

---

## 贡献

欢迎提交 Issue 和 Pull Request！
