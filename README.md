# IronHook

## Members

- [Yuqi Fan PB24000188](https://github.com/Rosaya-qwq)
- [Yifei Xiong PB24000048](https://github.com/USTC-XeF2)
- [Qifan Zhong PB24010467](https://github.com/C6-H14)
- [Anqiao Li PB24010490](https://github.com/Kurisu934)
- [Jiawen Liang PB24000358](https://github.com/juicyname)

## Project Introduction

IronHook is a high-performance, security-grade inline hook library designed specifically for the Android ARM64 architecture.

IronHook aims to solve the memory safety risks and complex call chain management problems of traditional C hook frameworks in multi-threaded environments by leveraging Rust's ownership model and RAII (Resource Acquisition and Initialization) mechanism.

## Schedule
 | **项目阶段**   | **日期** | **项目进展**                                                 | **工作安排**                                                 |
| -------------- | -------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| **选题确认**   | 3月23日  | 与指导老师沟通并得到明确同意，正式确认项目大作业选题为“使用 Rust 语言改写字节跳动开源的 `android-inline-hook` 项目”。 | 明确项目最终方向与总体目标，小组成员准备结束选题阶段，转入针对该开源项目的底层源码调研。 |
| **初步调研**   | 4月7日   | 开展了线上会议，针对 `android-inline-hook` 原始项目的架构展开初步讨论，计划具体调研其底层实现细节与工作机制。 | 小组安排分工，深入阅读并解析原始项目源码，重点梳理其 C/C++ 核心逻辑与 Hook 机制原理，为后续的 Rust 移植奠定基础。 |
| **可行性调研** | 4月13日  | 开展了线下会议，基于前期的源码调研结果，小组讨论后决定采取渐进式重构策略，优先从最核心的 Hook 层入手进行改写尝试。 | 1. **小组分工撰写可行性分析报告**（涵盖技术路线、难点分析等）； 2. 准备搭建 Rust 交叉编译与测试环境，针对 Hook 层的初步改写路线开展技术验证。 |