export const SHARE_QUERY_KEY = "s";

export const SNIPPETS = [
  {
    label: "Herllo World!",
    value: `"Herllo" + " " + "World!"`,
  },

  {
    label: "数值计算",
    value: `(10 + 2) * 30 + 5`,
  },

  {
    label: "姐妹们觉得呢",
    value: `姐妹们觉得呢 (那么普通却那么自信) {
  "Herllo";
} 我接受不等于我同意 {
  "unreachable";
}`,
  },

  {
    label: "拼单",
    value: `我认为 爱马仕 = 10;
我认为 LV = 15;

爱马仕 拼单 LV;`,
  },

  {
    label: "我接受不等于我同意",
    value: `我认为 factorial = 想要你一个态度(n) {
  姐妹们觉得呢 (n 我接受 0) {
    反手举报 1
  } 我接受不等于我同意 {
    反手举报 n * factorial(n - 1)
  }
}

factorial(5)`,
  },
];
