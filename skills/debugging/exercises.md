# 练习：调试

对应课程第 4 讲的练习（节选，按主题归入本 skill），复习用。

1. **调试归并排序**。下面的伪代码实现了归并排序但有一个 bug。用你喜欢的语言实现它，然后用调试器（gdb、lldb、pdb 或 IDE 调试器）找到并修复。测试向量：`merge_sort([3, 1, 4, 1, 5, 9, 2, 6])` 应返回 `[1, 1, 2, 3, 4, 5, 6, 9]`。在 merge 函数里设断点、单步，找到选错元素的位置。

   ```
   function merge_sort(arr):
       if length(arr) <= 1:
           return arr
       mid = length(arr) / 2
       left = merge_sort(arr[0..mid])
       right = merge_sort(arr[mid..end])
       return merge(left, right)

   function merge(left, right):
       result = []
       i = 0, j = 0
       while i < length(left) AND j < length(right):
           if left[i] <= right[j]:
               append result, left[i]
               i = i + 1
           else:
               append result, right[i]
               j = j + 1
       append remaining elements from left and right
       return result
   ```

2. **rr 反向调试找内存写坏**。把下面程序存为 corruption.c，`gcc -g corruption.c -o corruption` 编译运行——Student 1 的 ID 被破坏，但破坏发生在一个只碰 Student 0 的函数里。用 `rr record ./corruption` 和 `rr replay` 找元凶：对 `students[1].id` 设 watchpoint，破坏发生后用 reverse-continue 精确定位是哪一行覆盖了它。

   ```c
   #include <stdio.h>

   typedef struct {
       int id;
       int scores[3];
   } Student;

   Student students[2];

   void init() {
       students[0].id = 1001;
       students[0].scores[0] = 85;
       students[0].scores[1] = 92;
       students[0].scores[2] = 78;

       students[1].id = 1002;
       students[1].scores[0] = 90;
       students[1].scores[1] = 88;
       students[1].scores[2] = 95;
   }

   void curve_scores(int student_idx, int curve) {
       for (int i = 0; i < 4; i++) {
           students[student_idx].scores[i] += curve;
       }
   }

   int main() {
       init();
       printf("=== Initial state ===\n");
       printf("Student 0: id=%d\n", students[0].id);
       printf("Student 1: id=%d\n", students[1].id);

       curve_scores(0, 5);

       printf("\n=== After curving ===\n");
       printf("Student 0: id=%d\n", students[0].id);
       printf("Student 1: id=%d\n", students[1].id);

       if (students[1].id != 1002) {
           printf("\nERROR: Student 1's ID was corrupted! Expected 1002, got %d\n",
                  students[1].id);
           return 1;
       }
       return 0;
   }
   ```

3. **AddressSanitizer 找 use-after-free**。把下面程序存为 uaf.c。先不加 sanitizer 编译运行（`gcc uaf.c -o uaf && ./uaf`）——它看起来能正常工作。再用 `gcc -fsanitize=address -g uaf.c -o uaf && ./uaf` 编译运行，读错误报告：ASan 发现了什么 bug？修复它。

   ```c
   #include <stdlib.h>
   #include <string.h>
   #include <stdio.h>

   int main() {
       char *greeting = malloc(32);
       strcpy(greeting, "Hello, world!");
       printf("%s\n", greeting);

       free(greeting);

       greeting[0] = 'J';
       printf("%s\n", greeting);

       return 0;
   }
   ```

4. **strace 看系统调用**。用 strace（Linux）或 dtruss（macOS）追踪 `ls -l` 这类命令发起的系统调用——它调了哪些？再追踪一个更复杂的程序，看它打开了哪些文件。
5. **用 LLM 解释天书报错**。复制一条编译错误（C++ 模板或 Rust 的尤其合适），让 LLM 解释并给出修复；再把 strace 或 AddressSanitizer 的部分输出喂给它试试。

> 改编自 MIT The Missing Semester 课程 Lecture 4: Debugging and Profiling（讲义 + 口播稿，CC BY-NC-SA 4.0）：https://creativecommons.org/licenses/by-nc-sa/4.0/ · 课程站点：https://missing.csail.mit.edu/ · 讲座视频：https://www.youtube.com/watch?v=8VYT9TcUmKs
