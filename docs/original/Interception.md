**拦截战（Interception）** 是另一个复杂任务，其特点是两层嵌套的 pipeline_override 决定走完全不同的两条路径，加上 JumpBack 在路径内部的循环调度。

---

## 拦截战（Interception）完整操作流程

### 第一步：进入拦截战

任务启动后，程序检查当前是否在大厅界面——如果不在，先导航回大厅，导航完成后**回到这里继续**（JumpBack）。确认在大厅后，在方舟界面找到"拦截战"按钮，点击进入。

进入后，如果页面还没完全加载，就点击空白处等待，点完**回到这里继续等**（JumpBack），直到左上角出现"剩余拦截次数"字样，确认已进入拦截战主页面。 [1](#4-0) 

---

### 第二步：根据配置走两条完全不同的路径

程序确认进入主页面后，检查要执行哪种拦截战。

> **此处 pipeline_override 生效（第一层）**：用户在配置中选择了"普通拦截战"还是"异常个体拦截战"。选了哪个，对应节点就被启用（`enabled: true`），另一个被禁用（`enabled: false`）。两条路径只会走一条。 [2](#4-1) [3](#4-2) 

---

## 路径 A：普通拦截战

### 第三步（普通）：进入关卡并选择难度

程序找到"拦截战"按钮，点击（点击位置在按钮上方的关卡图片区域，向上偏移 50 像素）。进入关卡选择页面后，程序检查要选哪个难度：

> **此处 pipeline_override 生效（第二层，嵌套在普通拦截战下）**：用户选了 D 级、S 级还是特殊目标拦截战，对应的难度选择节点才被启用，其余两个保持禁用。

- 如果选了 D 级，找到"LEVEL D"文字，点击选中，点完**回到这里继续**（JumpBack）
- 如果选了 S 级，找到"LEVEL S"文字，点击选中，点完**回到这里继续**（JumpBack）
- 如果选了特殊目标，找到"特殊目标拦截战"文字，点击选中，点完**回到这里继续**（JumpBack）

难度选好后，直接开始战斗。 [4](#4-3) [5](#4-4) 

---

## 路径 B：异常个体拦截战

### 第三步（异常）：在列表中找到目标异常个体

程序找到"异常个体"按钮，点击（点击位置向右偏移 30、向上偏移 50 像素）。进入异常个体选择页面后，程序开始在列表中寻找用户指定的目标。

> **此处 pipeline_override 生效（第二层，嵌套在异常拦截战下）**：用户选了克拉肯、镜像容器、茵迪维利亚、死神还是过激派，只有对应的那一个目标节点被启用。

程序依次检查 5 个目标节点（只有一个被启用）：
- 如果当前页面显示的就是目标（比如"克拉肯"），直接进入下一步
- 如果不是，就点击右侧的"下一个"箭头切换到下一个异常个体，切完**回到这里继续找**（JumpBack），直到找到目标为止 [6](#4-5) [7](#4-6) [8](#4-7) 

### 第四步（异常）：挑战目标

找到目标异常个体后，点击"挑战"按钮，等待 200 毫秒页面稳定。

- 如果弹出"无法战斗"提示（说明今日拦截次数已耗尽），任务直接结束
- 否则，进入挑战页面，确认页面显示"异常"字样后，开始战斗，战斗结束后**回到这里继续**（JumpBack） [9](#4-8) 

---

### 关于手动战斗模式

> **此处 pipeline_override 生效**：用户可以选择是否开启"手动战斗"。默认关闭，程序全自动打。开启后，`ManualInterceptionBattle` 节点被启用，它会反向监控战斗页面——当屏幕上**没有**出现"达成阶段"文字时（`inverse: true`），触发 focus 事件通知用户介入手动操作。 [10](#4-9) [11](#4-10) 

---

## 两条路径的结构对比

| | 普通拦截战 | 异常个体拦截战 |
|---|---|---|
| **第二层 pipeline_override** | 选难度（D/S/特殊目标），启用对应难度节点 | 选目标（5选1），启用对应目标节点 |
| **JumpBack 的作用** | 选完难度后回来，确认选中再开始战斗 | 切换异常个体后回来，继续找目标 |
| **战斗结束后** | 直接结束 | JumpBack 回到挑战页面确认完成 |

### Citations

**File:** assets/resource/pipeline/Interception/Interception.json (L1-38)
```json
{
    "InterceptionMain": {
        "next": [
            "ArkEnterInterception",
            "[JumpBack]HallEnterArk"
        ]
    },
    "ArkEnterInterception": {
        "desc": "进入拦截战",
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    512,
                    585,
                    85,
                    23
                ],
                "replace": [
                    [
                        "载",
                        "截"
                    ]
                ],
                "expected": [
                    "拦截战"
                ]
            }
        },
        "action": {
            "type": "Click"
        },
        "post_wait_freezes": 1000,
        "next": [
            "InterceptionBattleFlow",
            "ArkEnterInterception",
            "[JumpBack]CommonClickBlank"
        ]
```

**File:** assets/resource/pipeline/Interception/Interception.json (L57-72)
```json
    "InterceptionBattleFlow": {
        "desc": "已进入拦截战，执行拦截战流程",
        "recognition": {
            "type": "And",
            "param": {
                "all_of": [
                    "InterceptionMainPageEntered"
                ]
            }
        },
        "next": [
            "NormalInterception",
            "AnomalyInterception",
            "CommonEndTask"
        ]
    },
```

**File:** assets/resource/pipeline/Interception/Interception.json (L73-94)
```json
    "ManualInterceptionBattle": {
        "desc": "手动拦截战",
        "enabled": false,
        "inverse": true,
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    708,
                    620,
                    43,
                    17
                ],
                "expected": [
                    "达成阶段"
                ]
            }
        },
        "focus": {
            "Node.Recognition.Succeeded": "$focus.ManualInterceptionBattle.Succeeded"
        }
    }
```

**File:** assets/tasks/Interception.json (L19-53)
```json
            "type": "select",
            "label": "$option.InterceptionType.label",
            "cases": [
                {
                    "name": "NormalInterception",
                    "label": "$option.NormalInterception.label",
                    "pipeline_override": {
                        "NormalInterception": {
                            "enabled": true
                        },
                        "AnomalyInterception": {
                            "enabled": false
                        }
                    },
                    "option": [
                        "NormalInterceptionLevel"
                    ]
                },
                {
                    "name": "AnomalyInterception",
                    "label": "$option.AnomalyInterception.label",
                    "pipeline_override": {
                        "NormalInterception": {
                            "enabled": false
                        },
                        "AnomalyInterception": {
                            "enabled": true
                        }
                    },
                    "option": [
                        "AnomalyInterceptionTarget"
                    ]
                }
            ],
            "default_case": "AnomalyInterception"
```

**File:** assets/tasks/Interception.json (L55-88)
```json
        "NormalInterceptionLevel": {
            "type": "select",
            "label": "$option.NormalInterceptionLevel.label",
            "cases": [
                {
                    "name": "LevelDChoose",
                    "label": "$option.LevelDChoose.label",
                    "pipeline_override": {
                        "LevelDChoose": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "LevelSChoose",
                    "label": "$option.LevelSChoose.label",
                    "pipeline_override": {
                        "LevelSChoose": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "InterceptionEXChoose",
                    "label": "$option.InterceptionEXChoose.label",
                    "pipeline_override": {
                        "InterceptionEXChoose": {
                            "enabled": true
                        }
                    }
                }
            ],
            "default_case": "InterceptionEXChoose"
        },
```

**File:** assets/tasks/Interception.json (L89-139)
```json
        "AnomalyInterceptionTarget": {
            "type": "select",
            "label": "$option.AnomalyInterceptionTarget.label",
            "cases": [
                {
                    "name": "InterceptionAnomalyKraken",
                    "label": "$option.InterceptionAnomalyKraken.label",
                    "pipeline_override": {
                        "InterceptionAnomalyKraken": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "InterceptionAnomalyMirrorContainer",
                    "label": "$option.InterceptionAnomalyMirrorContainer.label",
                    "pipeline_override": {
                        "InterceptionAnomalyMirrorContainer": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "InterceptionAnomalyIndivilia",
                    "label": "$option.InterceptionAnomalyIndivilia.label",
                    "pipeline_override": {
                        "InterceptionAnomalyIndivilia": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "InterceptionAnomalyUltra",
                    "label": "$option.InterceptionAnomalyUltra.label",
                    "pipeline_override": {
                        "InterceptionAnomalyUltra": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "InterceptionAnomalyHarvester",
                    "label": "$option.InterceptionAnomalyHarvester.label",
                    "pipeline_override": {
                        "InterceptionAnomalyHarvester": {
                            "enabled": true
                        }
                    }
                }
            ],
            "default_case": "InterceptionAnomalyKraken"
```

**File:** assets/tasks/Interception.json (L141-163)
```json
        "ManualInterceptionBattle": {
            "type": "switch",
            "label": "$option.ManualInterceptionBattle.label",
            "description": "$task.ManualInterceptionBattle.description",
            "cases": [
                {
                    "name": "No",
                    "pipeline_override": {
                        "ManualInterceptionBattle": {
                            "enabled": false
                        }
                    }
                },
                {
                    "name": "Yes",
                    "pipeline_override": {
                        "ManualInterceptionBattle": {
                            "enabled": true
                        }
                    }
                }
            ]
        }
```

**File:** assets/resource/pipeline/Interception/NormalInterception.json (L1-41)
```json
{
    "NormalInterception": {
        "desc": "普通拦截战",
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    679,
                    683,
                    56,
                    26
                ],
                "replace": [
                    [
                        "载",
                        "截"
                    ]
                ],
                "expected": [
                    ".*拦截战.*"
                ]
            }
        },
        "action": {
            "type": "Click",
            "param": {
                "target_offset": [
                    0,
                    -50,
                    0,
                    0
                ]
            }
        },
        "next": [
            "[JumpBack]LevelDChoose",
            "[JumpBack]LevelSChoose",
            "[JumpBack]InterceptionEXChoose",
            "BattleStart"
        ]
    },
```

**File:** assets/resource/pipeline/Interception/AnomalyInterception.json (L1-36)
```json
{
    "AnomalyInterception": {
        "desc": "异常个体拦截战",
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    741,
                    686,
                    97,
                    22
                ],
                "expected": [
                    ".*异常个体.*"
                ]
            }
        },
        "action": {
            "type": "Click",
            "param": {
                "target_offset": [
                    30,
                    -50,
                    0,
                    0
                ]
            }
        },
        "next": [
            "InterceptionAnomalyKraken",
            "InterceptionAnomalyMirrorContainer",
            "InterceptionAnomalyIndivilia",
            "InterceptionAnomalyHarvester",
            "InterceptionAnomalyUltra",
            "[JumpBack]InterceptionNextAnomaly"
        ]
```

**File:** assets/resource/pipeline/Interception/AnomalyInterception.json (L59-104)
```json
    "InterceptionChallengeClick": {
        "desc": "点击挑战",
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    615,
                    569,
                    51,
                    22
                ],
                "expected": [
                    "挑战"
                ]
            }
        },
        "action": {
            "type": "Click"
        },
        "post_wait_freezes": 200,
        "next": [
            "BattleCannotBattle",
            "[JumpBack]InterceptionChallengeBattle"
        ]
    },
    "InterceptionChallengeBattle": {
        "desc": "挑战页面战斗",
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    487,
                    470,
                    102,
                    20
                ],
                "expected": [
                    ".*异常.*"
                ]
            }
        },
        "next": [
            "BattleStart",
            "CommonEndTask"
        ]
    },
```

**File:** assets/resource/pipeline/Interception/AnomalyInterception.json (L189-212)
```json
    "InterceptionNextAnomaly": {
        "desc": "切换异常个体",
        "recognition": {
            "type": "TemplateMatch",
            "param": {
                "roi": [
                    741,
                    523,
                    43,
                    37
                ],
                "template": [
                    "Interception/NextAnomaly1.png",
                    "Interception/NextAnomaly2.png",
                    "Interception/NextAnomaly3.png",
                    "Interception/NextAnomaly4.png",
                    "Interception/NextAnomaly5.png"
                ]
            }
        },
        "action": {
            "type": "Click"
        }
    }
```
