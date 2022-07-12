# cffc_plat_pub
运行在边缘小盒子(arm64)上的rust程序 

### 功能：
+ web管理后台
+ 接受设备推送的数据，调用api进行处理
+ 存储数据(sqlite和本地文件)
+ 上传数据
+ 提供ws数据源

### 使用的crates:
+ actix-web
+ tokio
+ rusqlite
+ ...
