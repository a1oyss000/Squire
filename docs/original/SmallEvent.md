**小活动（SmallEvent）** 与 LargeEvent 共享了多个关键节点，整体结构高度相似，是最典型的复用案例。

---

## 复用节点一览

| 复用节点 | 复用方式 |
|---|---|
| `EventChallenge`（整个挑战 pipeline） | SmallEvent 和 LargeEvent 的挑战子任务都直接跳入此流程 |
| `NavigationEnterHall` | 两者的 Main 节点都用 `[JumpBack]NavigationEnterHall` 导航回大厅 |
| `BattleStart` / `BattleMainVisible` | 两者的战斗页面识别和战斗启动完全共用 |
| `DialoguesSkipStory` | 两者的 GoBack 节点和战斗页面都用 `[JumpBack]DialoguesSkipStory` 处理对话 |
| `CommonGoBack` | 两者的 GoBack 节点都用 `[JumpBack]CommonGoBack` 处理返回按钮 |
| `CommonConfirmReward` | 两者的任务领取流程都用 `[JumpBack]CommonConfirmReward` 确认奖励弹窗 |
| `CommonEndTask` | 两者的主页面轮询都以 `CommonEndTask` 结束 |

---

## 小活动（SmallEvent）完整操作流程

### 第一步：进入活动

任务启动后，程序先执行会员资格检查。然后检查当前是否在大厅界面——如果不在，先导航回大厅，导航完成后**回到这里继续**（JumpBack）。

> **此处复用 LargeEvent**：这里调用的 `NavigationEnterHall` 节点与 LargeEvent 完全相同，两个任务共用同一套导航回大厅的逻辑。

确认在大厅后，在屏幕右下角寻找活动入口图标。

> **此处 pipeline_override 生效**：程序认的图标图片取决于用户选择的主题。选了 B-Side Idol，就去认 `B-SideIdolLogo.png`；选了其他主题，则认对应图片。

找到图标后点击，等待进入剧情活动主页面。 [1](#5-0) [2](#5-1) 

---

### 第二步：剧情活动主页面——轮询调度子任务

程序确认左上角出现"剧情"字样，先点击一下屏幕左侧区域（确保页面焦点正确），等待页面稳定后，开始依次检查各个子任务。

> **此处 JumpBack 的核心逻辑**：与 LargeEvent 完全相同的调度机制——每次执行完一个子任务后，自动回到主页面重新检查，继续找下一个需要执行的子任务，直到所有子任务都无事可做，才最终结束。

> **此处 pipeline_override 生效**：挑战、剧情、任务三个子任务默认全部关闭。只有用户在配置中勾选了某项，pipeline_override 才将该节点的 `enabled` 改为 `true`，程序才真正去执行它。 [3](#5-2) [4](#5-3) 

---

### 第三步：挑战子任务

程序在主页面中找到"挑战"标签，点击进入挑战页面，等待 500 毫秒页面稳定。

然后跳入挑战流程，执行完成后**回到这里继续**（JumpBack），确认挑战已完成。

> **此处复用 LargeEvent**：这里调用的 `EventChallenge` 整个 pipeline 与 LargeEvent 的挑战子任务完全相同。进入挑战关卡页面后，程序找到标有"CHALLENGE"的未挑战关卡，点击进入战斗；如果没有未挑战关卡，就找标有"CLEAR"的已通关关卡，点击进入战斗。战斗结束后回到关卡列表继续找下一个，直到没有可打的关卡为止。

挑战完成后进入返回流程，**回到主页面轮询继续检查下一个子任务**（JumpBack）。 [5](#5-4) [6](#5-5) 

---

### 第四步：剧情子任务

程序在主页面中找到带有"加成"字样的关卡入口（即有加成 buff 的活动关卡），点击进入（点击位置向上偏移 50 像素，点击关卡图片区域而非文字）。

进入关卡列表页面后，等待页面稳定，开始两轮打关卡：

**第一轮：首次通关**

程序在关卡列表中轮询两种首次通关关卡：

- 找普通关卡按钮，找到就点击，点完**回到这里继续找**（JumpBack）
- 找困难关卡按钮，找到就点击，点完**回到这里继续找**（JumpBack）

> **此处 pipeline_override 生效**：程序认的关卡按钮图片取决于主题。选了 B-Side Idol，就去认 `B-SideIdolStageNormal.png` 和 `B-SideIdolStageHard.png`；选了其他主题，则认对应图片。

如果进入了战斗页面（检测到战斗界面出现）：

> **此处复用 LargeEvent**：战斗页面的识别（`BattleMainVisible`）、战斗启动（`[JumpBack]BattleStart`）、剧情对话跳过（`[JumpBack]DialoguesSkipStory`）三个节点与 LargeEvent 完全相同，共用同一套战斗处理逻辑。

- 开始战斗，战斗结束后**回到战斗页面继续等**（JumpBack）
- 如果途中弹出剧情对话，就跳过，跳完**回到战斗页面继续等**（JumpBack）
- 战斗彻底结束后，回到关卡列表页面继续找下一个可打的关卡

两种首次通关关卡都找不到了，进入第二轮。

**第二轮：扫荡**

同样轮询两种可扫荡关卡（普通可扫荡、困难可扫荡），每找到一个就点击，点完**回到这里继续找**（JumpBack）。

> **此处 pipeline_override 生效**：扫荡关卡的图片同样被主题替换。

两种扫荡关卡也都找不到了，进入返回流程，**回到主页面轮询**（JumpBack）。 [7](#5-6) [8](#5-7) 

---

### 第五步：任务子任务

程序在主页面中找到带有"任务"字样的入口，点击打开任务面板。确认面板已打开（有"全部"字样）后，开始循环：

- 点击"全部领取"按钮，点完**回到这里继续**（JumpBack）
- 如果弹出奖励确认窗口，点击确认，确认完**回到这里继续**（JumpBack）
- 如果弹出操作确认窗口，点击确认，确认完**回到这里继续**（JumpBack）

> **此处复用 LargeEvent**：`[JumpBack]CommonConfirmReward` 节点与 LargeEvent 的任务领取流程完全相同，共用同一套奖励确认逻辑。

如果发现"全部领取"按钮已变灰（颜色检测为灰色，说明全部领完），直接调用 `CommonGoBack` 退出任务面板，**回到主页面轮询**（JumpBack）。

> **此处复用 LargeEvent**：`CommonGoBack` 节点与 LargeEvent 完全相同。 [9](#5-8) 

---

### 返回机制（贯穿全程）

每个子任务结束后都会进入 `SmallEventGoBack`，流程与 LargeEvent 的 `LargeEventGoBack` 完全相同：

先检查是否已回到剧情活动主页面——如果已经在了，直接结束返回，**回到主页面轮询**（JumpBack）。如果还没回到：

> **此处复用 LargeEvent**：`[JumpBack]DialoguesSkipStory` 和 `[JumpBack]CommonGoBack` 两个节点与 LargeEvent 完全相同，共用同一套返回途中的对话处理和返回按钮点击逻辑。 [10](#5-9) 

---

## 与 LargeEvent 的主要差异

| 维度 | LargeEvent | SmallEvent |
|---|---|---|
| **子任务数量** | 4个（签到/挑战/剧情/任务）+ StarAnis专属 | 3个（挑战/剧情/任务），无签到 |
| **关卡种类** | Story1 + Story2普通 + Story2困难（3种） | 普通 + 困难（2种） |
| **pipeline_override 层数** | 最多3层（主题→内容→StarAnis子选项） | 2层（主题→内容） |
| **主题专属子任务** | StarAnis 有周边包包抽奖 | 无 |
| **任务面板** | 逐个点击红点领取 | 直接点"全部领取"循环 |

### Citations

**File:** assets/resource/pipeline/Event/SmallEvent/SmallEvent.json (L2-53)
```json
    "SmallEventMain": {
        "desc": "小活动",
        "action": {
            "type": "Custom",
            "param": {
                "custom_action": "MembershipCheck"
            }
        },
        "next": [
            "SmallEventStart",
            "[JumpBack]NavigationEnterHall"
        ]
    },
    "SmallEventStart": {
        "desc": "小活动开始",
        "recognition": {
            "type": "And",
            "param": {
                "all_of": [
                    "NavigationArkVisible"
                ]
            }
        },
        "next": [
            "SmallEventEntry",
            "CommonEndTask"
        ]
    },
    "SmallEventEntry": {
        "desc": "小活动入口",
        "recognition": {
            "type": "TemplateMatch",
            "param": {
                "roi": [
                    824,
                    571,
                    145,
                    68
                ],
                "template": [
                    "Common/RedDot.png" //占位，应该去task页面修改
                ]
            }
        },
        "action": {
            "type": "Click"
        },
        "next": [
            "SmallEventMainPageFlow",
            "SmallEventEntry"
        ]
    },
```

**File:** assets/resource/pipeline/Event/SmallEvent/SmallEvent.json (L71-104)
```json
    "SmallEventMainPageFlow": {
        "desc": "活动地区流程",
        "recognition": {
            "type": "And",
            "param": {
                "all_of": [
                    "SmallEventMainPageEntered"
                ]
            }
        },
        "action": {
            "type": "Click",
            "param": {
                "target": [
                    150,
                    650
                ]
            }
        },
        "post_wait_freezes": {
            "time": 500,
            "target": [
                446,
                539,
                389,
                82
            ]
        },
        "next": [
            "[JumpBack]SmallEventChallenge",
            "[JumpBack]SmallEventStory",
            "[JumpBack]SmallEventMission",
            "CommonEndTask"
        ]
```

**File:** assets/resource/pipeline/Event/SmallEvent/SmallEvent.json (L106-113)
```json
    "SmallEventGoBack": {
        "desc": "返回剧情活动页面",
        "next": [
            "SmallEventMainPageEntered",
            "[JumpBack]DialoguesSkipStory",
            "[JumpBack]CommonGoBack"
        ]
    }
```

**File:** assets/tasks/SmallEvent.json (L18-78)
```json
        "SmallEventTheme": {
            "type": "select",
            "default_case": "BSideIdol",
            "label": "$option.SmallEventTheme.label",
            "cases": [
                {
                    "name": "BSideIdol",
                    "label": "$option.SmallEventTheme.BSideIdol",
                    "pipeline_override": {
                        "SmallEventEntry": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "SmallEvent/B-SideIdol/B-SideIdolLogo.png"
                                    ]
                                }
                            }
                        },
                        "SmallEventStageNormalClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "SmallEvent/B-SideIdol/B-SideIdolStageNormal.png"
                                    ]
                                }
                            }
                        },
                        "SmallEventStageHardClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "SmallEvent/B-SideIdol/B-SideIdolStageHard.png"
                                    ]
                                }
                            }
                        },
                        "SmallEventStageNormalRepeatableClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "SmallEvent/B-SideIdol/B-SideIdolStageNormalRepeatable.png"
                                    ]
                                }
                            }
                        },
                        "SmallEventStageHardRepeatableClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "SmallEvent/B-SideIdol/B-SideIdolStageHardRepeatable.png"
                                    ]
                                }
                            }
                        }
                    }
                },
                {
                    "name": "Other",
                    "label": "$option.SmallEventTheme.Other"
                }
            ]
```

**File:** assets/tasks/SmallEvent.json (L80-116)
```json
        "SmallEventContent": {
            "type": "checkbox",
            "default_case": [
                "SmallEventChallenge",
                "SmallEventStory",
                "SmallEventMission"
            ],
            "label": "$option.SmallEventContent.label",
            "cases": [
                {
                    "name": "SmallEventChallenge",
                    "label": "$option.SmallEventContent.SmallEventChallenge",
                    "pipeline_override": {
                        "SmallEventChallenge": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "SmallEventStory",
                    "label": "$option.SmallEventContent.SmallEventStory",
                    "pipeline_override": {
                        "SmallEventStory": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "SmallEventMission",
                    "label": "$option.SmallEventContent.SmallEventMission",
                    "pipeline_override": {
                        "SmallEventMission": {
                            "enabled": true
                        }
                    }
                }
            ]
```

**File:** assets/resource/pipeline/Event/SmallEvent/SmallEventChallenge.json (L1-41)
```json
{
    "SmallEventChallenge": {
        "max_hit": 1,
        "timeout": 60000,
        "desc": "小活动挑战",
        "enabled": false,
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    485,
                    536,
                    308,
                    79
                ],
                "replace": [
                    [
                        "排",
                        "挑"
                    ]
                ],
                "expected": [
                    ".*挑战.*"
                ]
            }
        },
        "action": {
            "type": "Click"
        },
        "post_wait_freezes": 500,
        "next": [
            "[JumpBack]SmallEventChallengeStart",
            "SmallEventGoBack"
        ]
    },
    "SmallEventChallengeStart": {
        "max_hit": 1,
        "next": [
            "EventChallenge"
        ]
    }
```

**File:** assets/resource/pipeline/Event/Challenge/EventChallenge.json (L1-77)
```json
{
    "EventChallenge": {
        "desc": "活动挑战",
        "next": [
            "EventChallengeStagePage"
        ]
    },
    "EventChallengeStagePage": {
        "desc": "活动挑战页面",
        "recognition": {
            "type": "And",
            "param": {
                "all_of": [
                    "EventChallengeStageVisible"
                ]
            }
        },
        "post_wait_freezes": 500,
        "next": [
            "EventChallengeStage",
            "EventClearStage"
        ]
    },
    "EventChallengeStage": {
        "desc": "未挑战的关卡",
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    452,
                    253,
                    379,
                    424
                ],
                "expected": [
                    "(?i).*CHALLENGE.*"
                ],
                "order_by": "Vertical",
                "index": -1
            }
        },
        "action": {
            "type": "Click"
        },
        "post_wait_freezes": 500,
        "next": [
            "BattleStart",
            "EventChallengeStage"
        ]
    },
    "EventClearStage": {
        "desc": "已挑战的关卡",
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    452,
                    253,
                    379,
                    424
                ],
                "expected": [
                    ".*CLEAR.*"
                ],
                "order_by": "Vertical",
                "index": -1
            }
        },
        "action": {
            "type": "Click"
        },
        "post_wait_freezes": 500,
        "next": [
            "BattleStart",
            "EventClearStage"
        ]
    },
```

**File:** assets/resource/pipeline/Event/SmallEvent/SmallEventStory.json (L78-112)
```json
    "SmallEventEventStageBattleFlow": {
        "desc": "小活动活动关卡页面战斗流程",
        "next": [
            "SmallEventEventStageBattlePage",
            "[JumpBack]SmallEventStageNormalClick",
            "[JumpBack]SmallEventStageHardClick",
            "SmallEventEventStageQuickBattleFlow"
        ]
    },
    "SmallEventEventStageQuickBattleFlow": {
        "desc": "小活动活动关卡页面快速战斗流程",
        "next": [
            "SmallEventEventStageBattlePage",
            "[JumpBack]SmallEventStageNormalRepeatableClick",
            "[JumpBack]SmallEventStageHardRepeatableClick",
            "SmallEventGoBack"
        ]
    },
    "SmallEventEventStageBattlePage": {
        "desc": "小活动活动关卡页面战斗页面",
        "recognition": {
            "type": "And",
            "param": {
                "all_of": [
                    "BattleMainVisible"
                ]
            }
        },
        "next": [
            "[JumpBack]BattleStart",
            "SmallEventEventStagePageFlow",
            "[JumpBack]DialoguesSkipStory",
            "SmallEventGoBack"
        ]
    },
```

**File:** assets/resource/pipeline/Event/SmallEvent/SmallEventMission.json (L45-100)
```json
    "SmallEventMissionFlow": {
        "desc": "小活动任务流程",
        "recognition": {
            "type": "And",
            "param": {
                "all_of": [
                    "SmallEventMissionVisible"
                ]
            }
        },
        "next": [
            "SmallEventMissionClaimedFlow",
            "[JumpBack]SmallEventMissionClaimAll",
            "[JumpBack]CommonConfirmReward",
            "[JumpBack]CommonConfirmAction"
        ]
    },
    "SmallEventMissionClaimed": {
        "desc": "小活动任务奖励已领取",
        "recognition": {
            "type": "ColorMatch",
            "param": {
                "roi": [
                    680,
                    624,
                    137,
                    35
                ],
                "lower": [
                    105,
                    105,
                    105
                ],
                "upper": [
                    122,
                    122,
                    122
                ],
                "count": 100
            }
        }
    },
    "SmallEventMissionClaimedFlow": {
        "desc": "小活动任务奖励已领取流程",
        "recognition": {
            "type": "And",
            "param": {
                "all_of": [
                    "SmallEventMissionClaimed"
                ]
            }
        },
        "next": [
            "CommonGoBack"
        ]
    },
```
