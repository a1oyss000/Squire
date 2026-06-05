以下是融合了跳转逻辑的完整流程描述，在操作流程中自然体现 JumpBack 和 pipeline_override 的实际效果：

---

## 大活动（LargeEvent）完整操作流程

### 第一步：进入活动

任务启动后，程序先执行会员资格检查。然后检查当前是否在大厅界面——如果不在，就先导航回大厅，导航完成后**回到这里继续**（JumpBack）。确认在大厅后，程序在屏幕右下角寻找活动入口图标。

> **此处 pipeline_override 生效**：程序认的图标图片取决于用户选择的主题。选了 StarAnis，就去认 `StarAnisLogo.png`；选了其他主题，则认对应的图片。同一个"找图标"的动作，因配置不同而识别不同的图片。

找到图标后点击，等待进入活动地区主页面。 [1](#3-0) 

---

### 第二步：活动地区主页面——轮询调度子任务

程序确认左上角出现"活动地区"字样，等待页面稳定后，开始依次检查各个子任务。

> **此处 JumpBack 的核心逻辑**：主页面不是走完一遍就结束，而是每次执行完一个子任务后，**自动回到这里重新检查**，继续找下一个需要执行的子任务，直到所有子任务都无事可做，才最终结束。

检查顺序如下：

1. 签到能不能做？
2. 挑战能不能做？
3. 剧情能不能做？
4. 任务能不能做？
5. （StarAnis 主题且开启时）周边包包能不能做？

> **此处 pipeline_override 生效（两处）**：
> - 上面第 1~4 项，默认全部是**关闭状态**，程序遇到它们会直接跳过。只有用户在配置中勾选了某项，pipeline_override 才会将该节点的 `enabled` 改为 `true`，程序才真正去执行它。
> - 第 5 项本身的存在也是 pipeline_override 的结果：原本主页面只轮询 4 个子任务，用户选了 StarAnis 主题且开启"周边包包"后，pipeline_override 将整个轮询列表替换，第 5 个子任务被插入进来。 [2](#3-1) [3](#3-2) [4](#3-3) 

---

### 第三步：签到子任务

程序在主页面底部找到"签到"标签，点击进入签到页面。找到"全部领取"按钮，点击。

等待奖励弹窗出现——如果弹出了奖励确认窗口，就点击确认，确认完**回到这里继续等**（JumpBack），直到没有新的弹窗。

然后进入返回流程，回到活动地区主页面，**回到主页面轮询继续检查下一个子任务**（JumpBack）。 [5](#3-4) 

---

### 第四步：挑战子任务

程序在主页面底部找到"挑战"标签，点击进入挑战页面。

执行一次挑战战斗（调用通用挑战流程），战斗完成后**回到这里**（JumpBack），确认挑战已完成。

然后进入返回流程，**回到主页面轮询**（JumpBack）。 [6](#3-5) 

---

### 第五步：剧情子任务（最复杂）

程序在主页面底部找"STORY"标签。

> **此处 pipeline_override 生效**：找的是哪个 STORY 标签，取决于用户配置的剧情优先级。选了"Story2优先"，程序从标签列表末尾开始找（即优先最新的 Story2 剧情）；选了"Story1优先"，则从头开始找。

点击进入剧情活动页面，在页面中找到显示"剩余"字样的关卡入口，点击进入关卡列表。

**第一轮：首次通关**

程序进入关卡列表后，开始轮询三种首次通关关卡：

- 找 Story1 关卡按钮，找到就点击，点完**回到这里继续找**（JumpBack）
- 找 Story2 普通关卡按钮，找到就点击，点完**回到这里继续找**（JumpBack）
- 找 Story2 困难关卡按钮，找到就点击，点完**回到这里继续找**（JumpBack）

> **此处 pipeline_override 生效**：上面三种关卡按钮，程序认的图片取决于主题。选了 StarAnis，就去认 StarAnis 专属的关卡图片；Story2 困难关卡在 StarAnis 主题下，点击位置还会额外偏移（向右 70、向上 70 像素）。

如果进入了战斗页面（检测到战斗界面出现）：
- 开始战斗，战斗结束后**回到战斗页面继续等**（JumpBack）
- 如果途中弹出剧情对话，就跳过对话，跳完**回到战斗页面继续等**（JumpBack）
- 战斗彻底结束后，回到关卡列表页面，继续轮询下一个可打的关卡

三种首次通关关卡都找不到了，进入第二轮。

**第二轮：扫荡**

同样轮询三种可扫荡关卡（Story1 可扫荡、Story2 普通可扫荡、Story2 困难可扫荡），每找到一个就点击，点完**回到这里继续找**（JumpBack）。

> **此处 pipeline_override 生效**：扫荡关卡的图片同样被 StarAnis 主题替换。

三种扫荡关卡也都找不到了，进入返回流程，**回到主页面轮询**（JumpBack）。 [7](#3-6) [8](#3-7) 

---

### 第六步：任务子任务

程序在屏幕右侧找到有红点的任务按钮，点击打开任务面板。确认面板已打开（有"全部"字样）后，开始循环：

- 在面板内找有红点（可领取）的任务，点击
- 点击"全部领取"按钮，点完**回到这里继续**（JumpBack）
- 如果弹出奖励确认窗口，点击确认，确认完**回到这里继续**（JumpBack）
- 回到面板继续找下一个有红点的任务

如果发现"全部领取"按钮已变灰（说明全部领完），直接点击返回按钮退出任务面板，**回到主页面轮询**（JumpBack）。 [9](#3-8) 

---

### 第七步：StarAnis 周边包包子任务（仅 StarAnis 主题且开启时）

> 这个子任务能出现在轮询列表里，本身就是 pipeline_override 的结果（见第二步）。

程序进入周边包包页面，执行抽奖流程，完成后**回到主页面轮询**（JumpBack）。

---

### 返回机制（贯穿全程）

每个子任务结束后都会进入统一的返回节点，流程如下：

先检查是否已经回到活动地区主页面——如果已经在了，直接结束返回，**回到主页面轮询**（JumpBack）。

如果还没回到主页面：
- 如果途中弹出剧情对话，就跳过，跳完**回到这里继续返回**（JumpBack）
- 点击返回按钮，点完**回到这里继续检查**（JumpBack）
- 直到确认回到活动地区主页面为止 [10](#3-9)

### Citations

**File:** assets/resource/pipeline/Event/LargeEvent/LargeEvent.json (L30-53)
```json
    "LargeEventEntry": {
        "desc": "大活动入口",
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
            "LargeEventMainPageFlow",
            "LargeEventEntry"
        ]
    },
```

**File:** assets/resource/pipeline/Event/LargeEvent/LargeEvent.json (L71-97)
```json
    "LargeEventMainPageFlow": {
        "desc": "活动地区流程",
        "recognition": {
            "type": "And",
            "param": {
                "all_of": [
                    "LargeEventMainPageEntered"
                ]
            }
        },
        "pre_wait_freezes": {
            "time": 500,
            "target": [
                446,
                539,
                389,
                82
            ]
        },
        "action": "DoNothing",
        "next": [
            "[JumpBack]LargeEventLoginStamp",
            "[JumpBack]LargeEventChallenge",
            "[JumpBack]LargeEventStory",
            "[JumpBack]LargeEventMission",
            "CommonEndTask"
        ]
```

**File:** assets/resource/pipeline/Event/LargeEvent/LargeEvent.json (L99-106)
```json
    "LargeEventGoBack": {
        "desc": "返回活动地区",
        "next": [
            "LargeEventMainPageEntered",
            "[JumpBack]DialoguesSkipStory",
            "[JumpBack]CommonGoBack"
        ]
    }
```

**File:** assets/tasks/LargeEvent.json (L40-104)
```json
                        "LargeEventStory1StageClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "LargeEvent/StarAnis/StarAnisStory1Stage.png"
                                    ]
                                }
                            }
                        },
                        "LargeEventStory2StageNormalClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "LargeEvent/StarAnis/StarAnisStory2StageNormal.png"
                                    ]
                                }
                            }
                        },
                        "LargeEventStory2StageHardClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "LargeEvent/StarAnis/StarAnisStory2StageHard.png",
                                        "LargeEvent/StarAnis/StarAnisStory2StageHardSP.png"
                                    ]
                                }
                            },
                            "action": {
                                "param": {
                                    "target_offset": [
                                        70,
                                        -70,
                                        0,
                                        0
                                    ]
                                }
                            }
                        },
                        "LargeEventStory1StageRepeatableClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "LargeEvent/StarAnis/StarAnisStory1StageRepeatable.png"
                                    ]
                                }
                            }
                        },
                        "LargeEventStory2StageNormalRepeatableClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "LargeEvent/StarAnis/StarAnisStory2StageNormal.png"
                                    ]
                                }
                            }
                        },
                        "LargeEventStory2StageHardRepeatableClick": {
                            "recognition": {
                                "param": {
                                    "template": [
                                        "LargeEvent/StarAnis/StarAnisStory2StageHardRepeatable.png"
                                    ]
                                }
                            }
                        }
```

**File:** assets/tasks/LargeEvent.json (L113-159)
```json
        "LargeEventContent": {
            "type": "checkbox",
            "default_case": [
                "LargeEventLoginStamp",
                "LargeEventChallenge",
                "LargeEventStory",
                "LargeEventMission"
            ],
            "label": "$option.LargeEventContent.label",
            "cases": [
                {
                    "name": "LargeEventLoginStamp",
                    "label": "$option.LargeEventContent.LargeEventLoginStamp",
                    "pipeline_override": {
                        "LargeEventLoginStamp": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "LargeEventChallenge",
                    "label": "$option.LargeEventContent.LargeEventChallenge",
                    "pipeline_override": {
                        "LargeEventChallenge": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "LargeEventStory",
                    "label": "$option.LargeEventContent.LargeEventStory",
                    "pipeline_override": {
                        "LargeEventStory": {
                            "enabled": true
                        }
                    }
                },
                {
                    "name": "LargeEventMission",
                    "label": "$option.LargeEventContent.LargeEventMission",
                    "pipeline_override": {
                        "LargeEventMission": {
                            "enabled": true
                        }
                    }
                }
            ]
```

**File:** assets/tasks/LargeEvent.json (L161-184)
```json
        "LargeEventStarAnisMerchBagClick": {
            "type": "switch",
            "default_case": "Yes",
            "label": "$option.LargeEventStarAnisMerchBagClick.label",
            "cases": [
                {
                    "name": "Yes",
                    "pipeline_override": {
                        "LargeEventMainPageFlow": {
                            "next": [
                                "[JumpBack]LargeEventLoginStamp",
                                "[JumpBack]LargeEventChallenge",
                                "[JumpBack]LargeEventStory",
                                "[JumpBack]LargeEventMission",
                                "[JumpBack]LargeEventStarAnisMerchBagClick",
                                "CommonEndTask"
                            ]
                        }
                    }
                },
                {
                    "name": "No"
                }
            ]
```

**File:** assets/resource/pipeline/Event/LargeEvent/LargeEventLoginStamp.json (L28-52)
```json
    "LargeEventLoginStampClaim": {
        "desc": "大活动签到奖励领取",
        "recognition": {
            "type": "OCR",
            "param": {
                "roi": [
                    651,
                    649,
                    162,
                    37
                ],
                "expected": [
                    "全部领取"
                ]
            }
        },
        "action": {
            "type": "Click"
        },
        "post_wait_freezes": 1000,
        "next": [
            "[JumpBack]CommonConfirmReward",
            "LargeEventGoBack"
        ]
    }
```

**File:** assets/resource/pipeline/Event/LargeEvent/LargeEventChallenge.json (L1-41)
```json
{
    "LargeEventChallenge": {
        "max_hit": 1,
        "timeout": 60000,
        "desc": "大活动挑战",
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
            "[JumpBack]LargeEventChallengeStart",
            "LargeEventGoBack"
        ]
    },
    "LargeEventChallengeStart": {
        "max_hit": 1,
        "next": [
            "EventChallenge"
        ]
    }
```

**File:** assets/resource/pipeline/Event/LargeEvent/LargeEventStory.json (L134-169)
```json
    "LargeEventEventStageBattleFlow": {
        "desc": "大活动活动关卡页面战斗流程",
        "next": [
            "LargeEventEventStageBattlePage",
            "[JumpBack]LargeEventStory1StageClick",
            "[JumpBack]LargeEventStory2StageNormalClick",
            "[JumpBack]LargeEventStory2StageHardClick",
            "LargeEventEventStageQuickBattleFlow"
        ]
    },
    "LargeEventEventStageQuickBattleFlow": {
        "desc": "大活动活动关卡页面快速战斗流程",
        "next": [
            "LargeEventEventStageBattlePage",
            "[JumpBack]LargeEventStory1StageRepeatableClick",
            "[JumpBack]LargeEventStory2StageNormalRepeatableClick",
            "[JumpBack]LargeEventStory2StageHardRepeatableClick",
            "LargeEventGoBack"
        ]
    },
    "LargeEventEventStageBattlePage": {
        "desc": "大活动活动关卡页面战斗页面",
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
            "LargeEventEventStagePageFlow",
            "[JumpBack]DialoguesSkipStory",
            "LargeEventGoBack"
        ]
```

**File:** assets/resource/pipeline/Event/LargeEvent/LargeEventMission.json (L67-99)
```json
    "LargeEventClickMissionRedDot": {
        "desc": "大活动任务的红点",
        "recognition": {
            "type": "TemplateMatch",
            "param": {
                "roi": [
                    453,
                    106,
                    375,
                    59
                ],
                "template": [
                    "Common/PassRedDot.png"
                ]
            }
        },
        "action": {
            "type": "Click",
            "param": {
                "target_offset": [
                    -10,
                    10,
                    0,
                    0
                ]
            }
        },
        "next": [
            "LargeEventMissionClaimedFlow",
            "[JumpBack]LargeEventMissionClaimAll",
            "[JumpBack]CommonConfirmReward",
            "LargeEventClickMissionRedDot"
        ]
```
