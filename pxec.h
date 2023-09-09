#ifndef PXEC_H_
#define PXEC_H_

// beep package required
void beep(int freq, int len);
void print_version();
void remove_newline(char* line);
void clear_screen();
void save_to_file();
void run_cmd(char* in, char* argstr);
void remove_entry(char* in);
void add_entry(char* in, int* entry_count);
void list();
void print_list_entry(int i, int counter);
void edit();
void beep(int freq, int len);

#endif
