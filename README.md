# Change Notes

# Previous :
swooooooosh I forgot but I somehow wrote the code. basic kernel with a ramfs and simple shell works. I refactored the code directories on 24 Jan

# Jan 25 :

working on the user memory space and intgrating the syscalls. I faced a major wrong approach when I saw resources for remapping the existing kernel memory but it caused a page fault no matter what 
later figured out that separate memory address was to be allocated to the userspace. finally was able to implement ring0-ring3 transition using iretq stack frame ahhhh
