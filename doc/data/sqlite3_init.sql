create table be_user
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         varchar(50), /* '名字' */
    login_name   varchar(20) not null, /*  '登录名' */
    password     varchar(50) not null, /*  '登录密码' md5(md5(passwd)+salt) */
    salt         varchar(20) not null, /* 'md5的salt ' */
    token        varchar(100), /* 'token' */
    phone        varchar(50), /*  '手机号'*/
    email        varchar(100), /* '邮箱' */
    service_flag SMALLINT default 1, /*  '用户状态' */
    ref_count    SMALLINT default 0, /*  '在线数' */
    last_login   datetime, /* '最后登录时间' */
    token_expire datetime, /* 'token 失效时间' */
    memo         varchar(100), /* '备注' */
    gmt_create   datetime    not null, /* 创建时间 */
    gmt_modified datetime    not null /* 修改时间 */
);
create unique index idx_beuser_loginname on be_user (login_name);


create table cf_dfnode
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    node_sid     varchar(50)  not null, /* 节点uuid */
    name         varchar(50)  not null, /* 节点名称 */
    ip           varchar(50)  not null, /* 节点ip */
    url          varchar(200) not null, /* 节点调用url */
    node_type    SMALLINT     not null, /* 节点类型 1:analysis(采集模块) 2:recognition(识别模块) */
    sort_num     SMALLINT default 0, /* 排序用 */
    gmt_create   datetime     not null, /* 创建时间 */
    gmt_modified datetime     not null /* 修改时间 */
);
create unique index idx_dfnode_node_sid on cf_dfnode (node_sid);

create table cf_dfdb
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    db_sid       varchar(50) not null, /* db的uuid */
    name         varchar(50) not null, /* 节点名称 */
    node_sid     varchar(50) not null, /* db对应的dfnode的uuid */
    capacity     INTEGER     not null, /* 容量*/
    auto_match   SMALLINT    not null default 1, /* 是否参与自动匹配 1：参与, 0：不参与*/
    bw_flag      SMALLINT    not null default 1, /* 1:黑名单  2:白名单 */
    fp_flag      SMALLINT    not null default 1, /* 1:person库  2:facetrack库 */
    sort_num     SMALLINT             default 0, /* 排序用 */
    gmt_create   datetime    not null, /* 创建时间 */
    gmt_modified datetime    not null /* 修改时间 */
);
create unique index idx_dfdb_db_sid on cf_dfdb (db_sid);

create table cf_dfsource
(
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    src_sid          varchar(50)  not null, /* source 的uuid，在analysis模块中的uuid */
    name             varchar(50)  not null, /* 节点名称 */
    node_sid         varchar(50)  not null, /* source 对应的dfnode的uuid */
    src_url          varchar(200) not null, /* rtsp url */
    push_url         varchar(200) not null, /* 接收推送的url */
    ip               varchar(50)  not null, /* 摄像头对应的ip地址 */
    src_state        SMALLINT     not null default 1, /* 摄像头状态， 1：开启， 0：关闭 */
    src_config       text         not null, /* 摄像头配置参数 */
    grab_type        SMALLINT     not null default 1, /* 抓拍类型， 1：人脸， 2：车辆  3:人脸+车辆 */
    io_flag          SMALLINT     not null default 0, /*进出口标志 0:未知 1:进口 2:出口 3:进口/出口'*/
    direction        SMALLINT     not null default 0, /*'摄像头拍摄目标的方向 0:未知 1:正面 2:侧面 3:后面'*/
    tp_id            varchar(50), /* 第三方系统中的 id */
    upload_flag      SMALLINT     not null default 0, /* 是否上传数据， 1：开启， 0：关闭 */
    location_name    varchar(100), /* 点位名称 */
    resolution_ratio varchar(50), /* 分辨率 1920X1080 */
    coordinate       varchar(50), /* 坐标 23.34,23.34 */
    sort_num         SMALLINT     not null default 0, /* 排序用 */
    trip_line        integer      not null default 0, /* 水平触发线 */
    rtcp_utc         SMALLINT     not null default 0, /* rtcp中的ntp时间是否是utc时间， 1：是， 0：否 */
    lane_desc        varchar(500), /* 车道描述，lanes对象的json字符串 */
    lane_count       SMALLINT     not null default 4, /* 摄像头中，车道数量 */
    memo             varchar(200), /* 备注 */
    gmt_create       datetime     not null, /* 创建时间 */
    gmt_modified     datetime     not null /* 修改时间 */
);
create unique index idx_src_sid on cf_dfsource (src_sid);
create index idx_dfsource_node_sid on cf_dfsource (node_sid);
create index idx_dfsource_name on cf_dfsource (name);
create index idx_dfsource_sortnum on cf_dfsource (sort_num);

create table cf_poi
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    poi_sid       varchar(50)  not null, /* person uuid */
    db_sid        varchar(50)  not null, /* 所在db的sid */
    name          varchar(100) not null, /* 姓名 */
    gender        SMALLINT default 1, /*   0 不确定; 1 男性; 2 ⼥性 */
    identity_card varchar(50), /* 身份证 */
    threshold     SMALLINT     not null, /* 阈值 */
    tp_id         varchar(50), /* 第三方系统中的 id */
    feature_ids   varchar(400) not null, /* face ids,  faceid:quality,faceid:quality */
    cover         SMALLINT default 0, /* 是否有封面照 0：无，1：有 */
    tag           varchar(50), /* tag */
    imp_tag       varchar(50), /* imp tag */
    memo          varchar(100), /* 备注 */
    flag          SMALLINT default 0, /* flag */
    gmt_create    datetime     not null, /* 创建时间 */
    gmt_modified  datetime     not null /* 修改时间 */
);
create unique index idx_poi_sid on cf_poi (poi_sid);
create index idx_poi_db_sid on cf_poi (db_sid);
create index idx_poi_name on cf_poi (name);
create index idx_poi_gender on cf_poi (gender);
create index idx_poi_identity_card on cf_poi (identity_card);
create index idx_poi_threshold on cf_poi (threshold);
create index idx_poi_gmt_modified on cf_poi (gmt_modified);

create table cf_facetrack
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    ft_sid       varchar(50)  not null, /* facetrack uuid */
    src_sid      varchar(50)  not null, /* source 的uuid，在analysis模块中的uuid */
    img_ids      varchar(400) not null, /* face ids,  index:quality,index:quality */
    matched      SMALLINT              default 0, /* 是否匹配过 0：否，1：是 */
    judged       SMALLINT              default 0, /* 是否识别出某人 0：否，1：是 */
    alarmed      SMALLINT              default 0, /* 是否报警 0：否，1：是 */
    most_person  varchar(50), /* 匹配的 top1的person */
    most_score   double, /* 匹配的分值 */
    gender       SMALLINT              default 0, /*  0 不确定; 1 男性; 2 ⼥性 */
    age          SMALLINT              default 0, /* 年龄 */
    glasses      SMALLINT              default 0, /* 默认为0，0 不戴眼镜; 1 墨镜; 2 普通眼镜 */
    direction    SMALLINT              default 0, /* 0 未知；1 向上；2 向下 */
    plane_score  double                default -1, /* ⼈脸为平⾯的可能性, 0-1, 不确定时为-1, 分值高的不是人脸，分值低的是 */
    mask         SMALLINT              default 0, /* ⼝罩, 0 不确定; 1 戴⼝罩; 2 不戴⼝罩*/
    moustache    SMALLINT              default 0, /* 0：未知 ，1：有 ，2没有 */
    hat          SMALLINT              default 0, /* 0：未知 ，1：有 ，2没有 */
    tag          varchar(50), /* tag */
    flag         SMALLINT     not null default 0, /* flag 0:未上传 1:上传成功 2:上传失败 */
    db_flag      SMALLINT              default 0, /* 保存到识别模块的db中？ 0:未保存，1：已保存 */
    db_sid       varchar(50), /* 所在db的sid */
    feature_ids  varchar(400), /* face ids,  index:quality,index:quality */
    obj_id       varchar(50), /* GA1400基本对象统一标识, vcs datasourceid */
    submit_id    varchar(80), /* 提交上级平台返回的ID */
    submit_time  datetime, /* 提交上级平台时间 */
    capture_time datetime     not null, /* 抓拍时间 */
    gmt_create   datetime     not null, /* 创建时间 */
    gmt_modified datetime     not null /* 修改时间 */
);
create unique index idx_facetrack_ft_sid on cf_facetrack (ft_sid);
create index idx_facetrack_src_sid on cf_facetrack (src_sid);
create index idx_facetrack_judged on cf_facetrack (judged);
create index idx_facetrack_gender on cf_facetrack (gender);
create index idx_facetrack_alarmed on cf_facetrack (alarmed);
create index idx_facetrack_most_person on cf_facetrack (most_person);
create index idx_facetrack_capture_time on cf_facetrack (capture_time);

create table cf_dictory
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    group_label  varchar(50) not null, /* 键集合的名称，用来表示一个group包含一个或多个item */
    group_key    varchar(50) not null, /* 键集合的key，检索用，group只有一个item时，等同item_key */
    item_label   varchar(50) not null, /* 键的名称，显示用*/
    item_key     varchar(50) not null, /* 键的key，检索用 */
    item_value   text, /* 键的值 */
    sort_num     SMALLINT default 0, /* 排序用 */
    memo         varchar(200), /* 备注 */
    gmt_create   datetime    not null, /* 创建时间 */
    gmt_modified datetime    not null /* 修改时间 */
);
create unique index idx_dictory_itemkey on cf_dictory (item_key);
create index idx_dictory_groupkey on cf_dictory (group_key);

create table cf_delpoi
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    poi_id       INTEGER      not null, /* person id */
    poi_sid      varchar(50)  not null, /* person uuid */
    db_sid       varchar(50)  not null, /* 所在db的sid */
    name         varchar(100) not null, /* 姓名 */
    tp_id        varchar(50), /* 第三方系统中的 id */
    gmt_create   datetime     not null, /* 创建时间 */
    gmt_modified datetime     not null /* 修改时间 */
);
create index idx_delpoi_gmt_modified on cf_delpoi (gmt_modified);
create index idx_delpoi_dbsid on cf_delpoi (db_sid);

CREATE TRIGGER trg_cfpoi_del
    AFTER DELETE
    ON cf_poi
    FOR EACH ROW

BEGIN

    INSERT INTO cf_delpoi(poi_id, poi_sid, db_sid, name, tp_id, gmt_create, gmt_modified)
    VALUES (OLD.id, OLD.poi_sid, OLD.db_sid, OLD.name, OLD.tp_id, OLD.gmt_create,
            STRFTIME('%Y-%m-%d %H:%M:%f', 'now', 'localtime') || '+08:00');

END;

/* --- car table --- */
create table cf_cartrack
(
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    sid              varchar(50)  not null, /* facetrack uuid */
    src_sid          varchar(50)  not null, /* source 的uuid'*/
    img_ids          varchar(400) not null, /* index:quality,index:quality */
    alarmed          SMALLINT     not null default 0, /*  是否报警 0：否，1：是  */
    most_coi         varchar(50), /* 根据拍照匹配的coi */
    plate_judged     SMALLINT     not null default 0, /*车牌是否识别出来 0：否，1：是 */
    vehicle_judged   SMALLINT     not null default 0, /* 车型是否识别出来 0：否，1：是 */
    move_direct      SMALLINT     not null default 0, /* 运动⽅向，0 未知；1 向上；2 向下 */
    car_direct       varchar(50), /* ⻋辆⽅位*/
    plate_content    varchar(50), /* 车牌内容  */
    plate_confidence double, /* 车牌置信度  */
    plate_type       varchar(50), /* 车牌类型  */
    car_color        varchar(50), /* 车身颜色*/
    car_brand        varchar(50), /* 品牌 */
    car_top_series   varchar(50), /* 车系 */
    car_series       varchar(50), /* 车款 */
    car_top_type     varchar(50), /* 车粗分类别 */
    car_mid_type     varchar(50), /* 车类别 */
    tag              varchar(50), /* tag  */
    flag             SMALLINT     not null default 0, /* flag 0:未上传 1:上传成功 2:上传失败 */
    obj_id           varchar(50), /* GA1400基本对象统一标识, vcs datasourceid */
    submit_id        varchar(80), /* 提交上级平台返回的ID */
    submit_time      datetime, /* 提交上级平台时间 */
    is_realtime      SMALLINT     not null default 0, /* 是否是rtcp中时间 0:否 1:是 */
    capture_time     datetime     not null, /* 抓拍时间  */
    capture_ts       INTEGER      not null default 0, /* 抓拍时间 trip.real_time */
    capture_pts      INTEGER      not null default 0, /* 抓拍时间  trip.pts*/
    lane_num         SMALLINT     not null default 0, /* 计算出来的车道 编号，从中间到旁边，从1开始 */
    gmt_create       datetime     not null, /* 创建时间  */
    gmt_modified     datetime     not null /* 修改时间  */
);
create unique index idx_cf_cartrack_sid on cf_cartrack (sid);
create index idx_cf_cartrack_src_sid on cf_cartrack (src_sid);
create index idx_cf_cartrack_plate_judged on cf_cartrack (plate_judged);
create index idx_cf_cartrack_vehicle_judged on cf_cartrack (vehicle_judged);
create index idx_cf_cartrack_alarmed on cf_cartrack (alarmed);
create index idx_cf_cartrack_plate_content on cf_cartrack (plate_content);
create index idx_cf_cartrack_capture_time on cf_cartrack (capture_time);

create table cf_coi
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    sid           varchar(50) not null, /* car uuid */
    group_sid     varchar(50) not null, /* 所在group的sid */
    plate_content varchar(50) not null, /* 牌照号码 */
    plate_type    varchar(50), /* 车牌类型*/
    car_brand     varchar(50), /* 品牌 */
    car_series    varchar(50), /* 系列 */
    car_size      varchar(50), /* 车型尺寸 */
    car_type      varchar(50), /* 车型 */
    owner_name    varchar(50), /* 车主姓名 */
    owner_idcard  varchar(50), /* 车主身份证 */
    owner_phone   varchar(50), /* 车主电话 */
    owner_address varchar(100), /* 车主地址 */
    flag          SMALLINT    not null default 0, /* flag */
    tag           varchar(50), /* tag */
    imp_tag       varchar(50), /* imp tag */
    memo          varchar(100), /* 备注 */
    gmt_create    datetime(3) not null, /* 创建时间 */
    gmt_modified  datetime(3) not null /* 修改时间 */
);

create unique index idx_coi_sid on cf_coi (sid);
create unique index idx_coi_plate on cf_coi (plate_content);
create index idx_coi_groupsid on cf_coi (group_sid);
create index idx_coi_gmt_modified on cf_coi (gmt_modified);

create table cf_coi_group
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    sid          varchar(50) not null, /*  uuid */
    name         varchar(50) not null, /* 分组名称 */
    bw_flag      SMALLINT    not null default 1, /* 1:黑名单  2:白名单 */
    memo         varchar(100), /* 备注 */
    gmt_create   datetime(3) not null, /* 创建时间 */
    gmt_modified datetime(3) not null /* 修改时间 */
);

create unique index idx_coigroup_sid on cf_coi_group (sid);
create index idx_coi_group_sid on cf_coi (group_sid);

/* --- options table --- */

create table cf_gate
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    src_sid      varchar(50)  not null, /* 摄像头 的uuid */
    sid          varchar(50)  not null, /* uuid */
    name         varchar(50)  not null, /* 门禁名称 */
    uni_code     varchar(50), /* 门禁编号 */
    flag         SMALLINT default 1, /* 状态， 1：启用， 0：禁用 */
    ac_config    varchar(200) not null, /* 门禁控制板 SN|ipaddr|door|关门时长  423181561|192.168.1.225:60000|1|3 */
    ac_type      SMALLINT default 1, /* 门禁控制器类型，1:微耕 2:聚英*/
    sort_num     SMALLINT default 0, /* 排序用 */
    memo         varchar(200), /* 备注 */
    gmt_create   datetime     not null, /* 创建时间 */
    gmt_modified datetime     not null /* 修改时间 */
);
create unique index idx_gate_srcsid on cf_gate (src_sid);
create unique index idx_gate_sid on cf_gate (sid);

create table cf_gatehistory
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    ft_sid       varchar(50) not null, /* facetrack uuid */
    src_sid      varchar(50) not null, /* 摄像头 的uuid */
    src_name     varchar(50) not null, /* 摄像头名称 */
    gate_sid     varchar(50) not null, /* 门禁uuid */
    gate_name    varchar(50) not null, /* 门禁名称 */
    poi_sid      varchar(50) not null, /* 名单uuid */
    poi_name     varchar(50) not null, /* 名单姓名 */
    poi_idcard   varchar(50), /* 名单身份证 */
    gmt_create   datetime    not null, /* 创建时间 */
    gmt_modified datetime    not null /* 修改时间 */
);
create unique index idx_gatehistory_ft_sid on cf_gatehistory (ft_sid);
create index idx_gatehistory_src_sid on cf_gatehistory (src_sid);
create index idx_gatehistory_gate_sid on cf_gatehistory (gate_sid);
create index idx_gatehistory_poi_name on cf_gatehistory (poi_name);
create index idx_gatehistory_poi_sid on cf_gatehistory (poi_sid);

/* --- init data --- */

/* be_user  admin / admin */
insert into be_user(name, login_name, password, salt, memo, gmt_create, gmt_modified)
values ("admin", "admin", "fb7ab6e329190e7c88204e361b0b35a8", "d59a17c2-d815-439e-ba91-e87bc55b4748", "memo",
        datetime('now'), datetime('now'));

/* cf_dfnode */
insert into cf_dfnode(node_sid, name, ip, url, node_type, sort_num, gmt_create, gmt_modified)
values ("7d4f2f62-0f7e-4a80-882c-08fa65e700f0", "local_analysis", "localhost", "http://localhost:7001", 1, 0,
        datetime('now'), datetime('now'));

insert into cf_dfnode(node_sid, name, ip, url, node_type, sort_num, gmt_create, gmt_modified)
values ("3b08100a-d1a0-4a27-8c7d-4c6aa2caa0fa", "local_recognition", "localhost", "http://localhost:7002", 2, 0,
        datetime('now'), datetime('now'));

/* cf_dfdb */
insert into cf_dfdb(db_sid, name, node_sid, capacity, auto_match, bw_flag, fp_flag, sort_num, gmt_create, gmt_modified)
values ("926d00b1-ce50-41d6-9c66-4df096fec013", "路人库", "3b08100a-d1a0-4a27-8c7d-4c6aa2caa0fa", 50000, 0, 0, 2, 0,
        datetime('now'), datetime('now'));

insert into cf_dfdb(db_sid, name, node_sid, capacity, auto_match, bw_flag, fp_flag, sort_num, gmt_create, gmt_modified)
values ("56e6a47c-3d4d-4f99-b6a3-ca24028358df", "黑名单", "3b08100a-d1a0-4a27-8c7d-4c6aa2caa0fa", 150000, 1, 1, 1, 1,
        datetime('now'), datetime('now'));

insert into cf_dfdb(db_sid, name, node_sid, capacity, auto_match, bw_flag, fp_flag, sort_num, gmt_create, gmt_modified)
values ("aee4a866-bda7-4019-9c37-8b98a37e4ad5", "白名单", "3b08100a-d1a0-4a27-8c7d-4c6aa2caa0fa", 50000, 1, 2, 1, 2,
        datetime('now'), datetime('now'));


/* cf_coi_group */
insert into cf_coi_group(sid, name, bw_flag, memo, gmt_create, gmt_modified)
values ("55a1a073-aa94-408a-b891-47b429c9a161", "黑名单", 1, "车辆黑名单", datetime('now'), datetime('now'));

insert into cf_coi_group(sid, name, bw_flag, memo, gmt_create, gmt_modified)
values ("65df24e6-827a-46c7-8843-92c55aa76015", "白名单", 2, "车辆白名单", datetime('now'), datetime('now'));
