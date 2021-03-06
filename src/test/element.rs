use glayout;
use glayout::canvas::element::{Element, Empty, Image, Text};
use glayout::canvas::element::style::{DisplayType};
use super::super::utils::PretendSend;
use std::time;

pub fn init() {
    register_test_case!(module_path!(), rc_context, {
        let mut context = rc_context.borrow_mut();
        let pixel_ratio = context.device_pixel_ratio();
        context.set_canvas_size(800, 600, pixel_ratio);
        context.set_clear_color(0.5, 1., 0.5, 1.);

        let elem = {
            let cfg = context.canvas_config();
            let mut root = context.root().borrow_mut();
            cfg.append_style_sheet(&mut root, "
                .abs { position: absolute }
            ");
            let elem = element!(&mut root, &cfg, Empty {
                font_family: "serif, 宋体";
                Empty {
                    class: "abs";
                    left: 500.;
                    top: 100.;
                    width: 100.;
                    height: 100.;
                    background_color: (1., 0.5, 0.5, 1.);
                };
                Text {
                    class: "abs";
                    left: 10.;
                    top: 10.;
                    width: 50.;
                    set_text("Absolute Positioning");
                };
                color: (0., 0., 1., 0.5);
                Empty {
                    display: DisplayType::Block;
                    Text {
                        font_size: 24.;
                        set_text("LARGE TEXT");
                    };
                    Empty;
                    Image {
                        width: 400.;
                        height: 400.;
                        opacity: 0.8;
                        load("resources/test.png");
                    };
                    Empty {
                        Text {
                            font_size: 16.;
                            set_text(ARTICLE);
                        };
                        top: 750.;
                    };
                };
            });
            elem
        };
        let mut root_elem = context.root().borrow_mut();
        root_elem.append(elem);

        let rc_context = PretendSend::new(rc_context.clone());
        glayout::set_timeout(move || {
            rc_context.borrow_mut().redraw();
        }, time::Duration::new(1, 0));

        return 0;
    });
}

const ARTICLE: &str = "“有bug啊！”

听到鸭鸭的一声惊呼，我抄起手边的旧报纸小跑到客厅里。向四处望了一圈，却什么也没发现。

“它刚刚钻到沙发底下了！”鸭鸭半藏在我身后，一只手指着沙发的一个角落，另一只手偷偷拉着我的裙边。她比较害怕小虫子。

走近她指向的位置。正当我觉得毫无办法的时候——

“又出来了！”

她叫起来的同时，我已经将报纸卷挥了过去，死死按在地上。

可是，我扑了个空。它受到了惊吓，在乱窜着。

“呀——”

* * *

“do do so so la la so ——”

受到惊吓的同时，早晨的铃声也响了起来。我猛地按掉铃声，坐起身来。四周依旧是平日卧室的光景。

梦到了小时候的事情。而且那明明就不是bug，只是蟑螂而已。

“哈——”这算是职业病了吧。

我呆坐在床上。早晨的卧室里，几束光透过窗帘的缝隙，在地板上画下光亮的线条。是个有阳光的日子啊。

鸭鸭她，最近怎么样了呢？

胡思乱想着的时候，铃声又响了起来。我逃出被窝，快速重复着工作日早晨的洗抹穿戴。

从冰箱里取出一盒蛋糕。锁好门。出发！

* * *

安全了。在迈入公司写字楼的瞬间，我将步子慢了下来。

手里依旧捧着的蛋糕让自己感觉到了饥饿。

“啊。”拆开蛋糕盒的时候，发现里面并没有叉子。没办法，只好一口咬了下去。

感觉到蛋糕上的奶油粘在了嘴唇上——好像还粘在了脸上——也有可能是错觉吧。

蛋糕吃掉一半的时候，我已经到达了自己的座位。刚刚坐定时，隐约听到了几个座位外的议论声。

“这个，要让璃姐修复一下了。”

糟糕，听到了什么不妙的东西。一大早就听到自己的别称，我一紧张，从剩余的蛋糕上咬下了一大口。

接着，小荫带着明亮的声线快步过来。

“璃姐——”

* * *

程序员的日常任务就是与bug的战斗。

这也是大家各显神通的时候。比如佑哥的神技“药圣的纲目”，可以根据前人的历史记录对症下药；还有依姐“夏洛克的放大镜”和佳哥“华生的手术刀”，二人合作无往不利。据说一些同事还有不为人知的秘技。

而我只会最普通的“粘虫贴大法”。

打开电脑，错落的英文字符映入眼帘。我闯进代码世界里。

这是人们努力构筑着的、五彩斑斓的幻想世界。

* * * 代码世界 * * *

今天需要解决的是一群零乱的小虫子。它们已经闻风而逃，躲在难以察觉的角落里。对付它们，最困难的工作是找到它们的藏身之处、抓住它们。

“一群胆小的家伙！”我的自言自语当然没有得到回应。

于是，我在它们经常出没的关键之处布上了粘虫贴。这种粘虫贴具有强大的粘性，只要一碰到，它们就难以挣脱，只能等待我将它们肢解。

接着我拿起扫把，将各处仔细清扫一遍。受到惊吓的虫子窜出来——当然，它们逃不过我的陷阱！

这便是最常见的套路“粘虫贴大法”。它也有个帅气的名字“除虫者”（debugger）。

我得意洋洋地看着被抓住的虫子们。

“噫——你笑得好阴险啊！”

发现小荫也来到了我旁边。我猛然意识到自己的笑容有些残虐。

调整了自己的嘴角，我指着脚下：“看这！”

粘虫贴上已经粘上了许多小虫子。它们挣扎着，然而那都是徒劳。已经逃不出我的掌心了！

“噫——”小荫很讨厌它们。

她手里拿着几张A4纸，上面画着虫子的图鉴。纸张上方印着英文标题“buglist”。她看几眼虫子又看几眼图鉴，还拿着红色的笔在图鉴上打着标注。

“快把它们清理掉啦！”小荫催促着。

我将粘住的虫子一只一只肢解。它们不应属于这个世界。

“还差一只。”小荫说着，将图鉴递给我。图鉴上有一行被加了星号。看样子只有它还藏匿着。

它的长相有些奇怪，和我在这里见过的虫子都不太像。不过，看起来就不像很难对付的那种。

“只剩它了吗？那它逃不了多久了！”

“璃姐真厉害！”小荫敬佩地说着，“那我去忙别的事情啦！”

然而，小荫再次回到这里的时候，事情并没有什么进展。

即使我铺满粘虫贴，虫子依然无迹可寻。

“原来，这是最终boss吗！”我意识到事情没有这么简单。

“哪有藏得这么好的最终boss啊！”

“不，这也是最终boss的一种。”而且，恰恰是最难对付的那种。

只要能与boss相遇，就有能战胜它的机会；若是它完美地躲藏着，那就真的束手无策了。

我并没有见过这样的boss。可是，想起之前佳哥讲述过的一段类似经历，我仿佛看到了他恼怒的样子。

“啊啊啊啊啊啊——”

就这么僵持着，过了下班的时间。

“还是明天再想办法吧。说不定佳哥或者依姐有好主意。”

疲劳积攒了一整天。只能暂时撤退了。

";
