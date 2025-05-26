/**
 * main.c
 * @copyright (C) 2021 SML# Development Team.
 * @author UENO Katsuhiro
 */
/* 
#include "smlsharp.h"

void sml_main(void);
void sml_load(void *);

int
main(int argc, char **argv)
{
	sml_init(argc, argv);
	sml_gcroot_load(&(void(*)(void *)){sml_load}, 1);
	sml_main();
	sml_exit(0);
}
*/

use libc::{c_char, c_int};

/* 
extern "C"{
        fn sml_init(argc:c_int,argv:*const *const c_char);
    fn sml_main();
    fn sml_load(); 
    fn sml_exit(i:c_int);
}
*/
fn main(){
/* 
    unsafe { sml_init(argc,argv) };
    sml_gcroot_load(sml_load,1);
    unsafe { sml_main() };
    unsafe { sml_exit(0) };
*/
}