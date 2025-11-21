# FondsPod开发说明书

FondsPod是一个基于档案管理思想的开源文档管理工具。 它旨在帮助用户高效地组织、存储和检索各种类型的文档和文件。本文档将详细介绍FondsPod的开发计划、功能模块以及技术实现细节。

## 基本概念

FondsPod采用档案管理的核心思想，将文档分为以下四个层级：

- 全宗（Fonds）：
- 案卷（Series）：
- 文件（File）：
- 件（Item）：

- 档案库：档案库是全宗的集合，用户可以创建和管理多个档案库。每个档案库可以包含多个全宗。每个档案库关联一个本地文件目录，程序建立关联后，会在目录创建一个sqlite数据库`.fondspod.db`并初始化表，用于存储档案库的元数据。档案库本身的数据存储在用户目录的FondsPod文件中。


### SQLite表结构

:::mermaid
erDiagram
    Schemas {
        string schemaNo "schemaNo"
        string name "Name"
    }
    SchemaItems {
        string schemaNo FK "Schema No"
        string itemNo "Item No"
        string itemName "Item Name"
    }
    Schemas ||--o{ SchemaItems : has

    Fonds {
        string fondNo "Fond No"
        string fondClassificationCode "Fond ClassificationCode"
        time createdAt "Created At"
    }
    Series {
        string seriesNo "Series No"
        string fondNo FK "Fond No"
    }
    Files {
        string fileNo "File No"
        string seriesNo FK "Series No"
    }
    Items {
        string itemNo "Item No"
        string fileNo FK "File No"
    }

    FondSchemas {
        string fondNo FK "Fond No"
        string schemaNo FK "Schema No"
        string orderNo "Order No"
    }

    FondClassifications {
        string code "Code"
        string name "Name"
        string parentId FK "Parent ID"
        bool isActive "Is Active"
    }

    Fonds ||--o{ Series : contains
    Series ||--o{ Files : contains
    Files ||--o{ Items : contains

    Sequences {
        string prefix "Prefix"
        int currentValue "Current Value"
    }
:::


## 功能模块

首页左侧导航栏包含以下功能模块：
- 档案库
- Schema
- 设置

### 编号生成（纯后台函数）

编号生成模块负责为全宗、案卷、文件和件生成唯一的编号。编号格式为前缀+流水号，编号信息存储在Sequences表中。生成函数可以指定流水号位数，缺省为2位。


### 案卷Schema管理

静态的案卷Schema管理模块，允许用户定义和管理案卷的元数据结构。Schema有两级，左侧为Schema列表，右侧为选中Schema的Item列表。提供添加、删除、修改Schema和Item的功能。有全宗在使用Schema时禁止修改和删除。

特殊的，初始化时会插入一条No和Name均为Year的Schema，此schema不能删除和修改，也不能添加SchemaItem。该Schema用于年度维度分类，如2020、2021等，在创建全宗时动态生成年份项。

### 设置

设置以配置文件的形式保存到用户目录中。
- 主题配置：允许用户选择应用程序的主题（如浅色模式、深色模式等）。
- 语言配置：允许用户选择应用程序的显示语言（如中文、英文等），程序要支持多语言切换。
- 档案库列表：允许用户设置和修改档案库的默认存储路径，档案库有路径和名字两个字段，名字默认为目录名。
- 添加档案库：选择本地目录，创建新的档案库，档案库名默认为目录名，此操作会更新配置文件中的档案库列表。


### 档案库与全宗(Fonds)管理

档案库列表以用户配置文件的形式存储。界面工具栏中以下拉的形式展示档案库，选择后展示该档案库下的全宗列表，默认选择上次打开的档案库(每次打开档案库时会报错到配置文件中)。

- 展示全宗列表：以列表的形式展示当前档案库中的所有全宗。
- 添加全宗：创建新的全宗，填写全宗相关元数据：
    - FondClassification（全宗分类），必填；
    - FondSeriesSchemas（全宗案卷Schema），必填，可多选，用于定义该全宗下案卷的元数据结构；
    - CreatedAt（创建时间），必填，默认为当前时间。
- 删除全宗
- 打开全宗：进入选中的全宗，展示该全宗下的案卷列表。

添加全宗后，有一下附加逻辑：
- 生成编号：以[FondClassificationCode][两位流水号]
- 生成FondSchemas记录：为该全宗关联所选的案卷Schema
- 更新案卷：根据FondSchemas生成Series。每个Sechma的Item生成笛卡尔积作为Sierie，例如，选择了Year和Department两个Schema，Year有2020、2021两个Item，Department有HR、IT两个Item，则会生成4个Series，分别为2020-HR、2020-IT、2021-HR、2021-IT。拼接顺序按照orderNo升序排列。特殊地，Year没有Items，取CreateAt到当前年份作为SchemaItems。Series的seriesNo为各Item的itemNo用“-”连接而成。

### 文件(Files)管理
文件管理界面同时展示案卷（Series）和其下的文件（Files）。用左右结构展示，左侧列出案卷列表，默认选中第一个案卷，右侧展示选中案卷下的文件列表。

工具栏有以下功能：
- 更新案卷：复用全宗管理中的更新案卷逻辑（不要删除任何案卷）。
- 添加文件：为选中的案卷添加文件，填写文件相关元数据，生成文件编号，编号格式为[FondNo]-[SeriesNo]-[两位流水号]。此操作会在本地文件系统中创建对应的文件夹。
- 打开文件夹：双击已添加的文件，会在文件管理器中打开选中的文件对应的本地文件夹。文件夹路径为[档案库路径]/[文件编号]。
- 删除文件：删除选中的文件记录，将对应文件夹移动到`[档案库路径]/.trash/`目录下，避免误删。