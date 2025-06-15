A guide to default editor commands.
| Command                      | Description                                                       | Args                  |
|------------------------------|-------------------------------------------------------------------|-----------------------|
| [!hi](#hi)                   | Prints "hi"                                                       | -                     |
| [!w](#w)                     | Writes current tab buffer to a file (filename argument is optional)| filename              |
| [!r](#r)                     | Reads file into current tab; warns if there are unsaved changes    | filename              |
| [!ri](#ri)                   | Reads file into current tab, even with unsaved changes             | filename              |
| [!rn](#rn)                   | Reads file into a new tab                                          | filename              |
| [!q](#q)                     | Quits the editor; warns if there are unsaved changes               | -                     |
| [!qi](#qi)                   | Quits the editor, even with unsaved changes                        | -                     |
| [!exec](#exec)               | Executes a command; shows stdout in the status bar                 | command               |
| [!execn](#execn)             | Executes a command; shows stdout in a new tab                      | command               |
| [!exec_f](#exec_f)           | Executes a script/executable file; shows stdout in the status bar  | filename              |
| [!execn_f](#execn_f)         | Executes a script/executable file; shows stdout in a new tab       | filename              |
| [!version](#version)         | Prints the current editor version in the status bar                | -                     |
| [!tab](#tab)                 | Tabs manager utility                                               | -                     |
| [!tab new](#tab-new)         | Creates a new tab                                                  | -                     |
| [!tab goto](#tab-goto)       | Switches to tab by its number                                      | num                   |
| [!tab rm](#tab-rm)           | Closes tab by its number                                           | num                   |
| [!tab next](#tab-next)       | Opens the next tab                                                 | -                     |
| [!tab prev](#tab-prev)       | Opens the previous tab                                             | -                     |
| [!tab rename](#tab-rename)   | Renames the specified tab                                          | [num] [new name]      |
| [!alias](#alias)             | Aliases manager                                                    | -                     |
| [!alias new](#alias-new)      | Creates new alias                                                 | [alias_name] [com]      |
| [!alias rm](#alias-rm)        | Removes existing alias                                            | [alias_name]          |


## !hi
A dummy command prints "hi"
Args: -
## !w
Writes buffer of current tab into file with name in first arg.
Writes it into file already opened in current tab by default (if no args passed).
Args: filename
## !r
Reads file with name from first arg into current tab.
Throws warning if cur tab has unsaved changes.
Args: filename
## !ri
Reads file with name from first arg into current tab
even if cur tab has unsaved changes.
Args: filename
## !rn
Reads file with name from first args into new tab.
Args: filename
## !q
Quits the editor.
Throws warning if unsaved changes.
Args: -
## !qi
Quits the editor even with unsaved changes.
Args: -
## !exec
Executes the command.
By default, command will be executed in:
sh (unix-like os)
cmd (windows)
Prints stdout output into status bar.
Args: command
## !execn
Executes the command.
By default, command will be executed in:
sh (unix-like os)
cmd (windows)
Prints stdout output into new tab.
Pass ~cur as first argument in order to print result into current tab instead new one.
Args: command
## !exec_f
Executes the script/executable file.
By default, file will be executed in:
sh (unix-like os)
cmd (windows)
Prints stdout output into status bar.
Args: filename
## !execn_f
Executes the script/executable file.
By default, file will be executed in:
sh (unix-like os)
cmd (windows)
Prints stdout output into new tab.
Pass ~cur as first argument in order to print result into current tab instead new one.
Args: filename
## !version
Prints current editor version into status bar
Args: -
## !tab
Tabs manager utility.
### !tab new
Creates new tab
Args: -
### !tab goto
Opens tab with specified ordinal number.
Works with only opened tabs
Args: num
### !tab rm
Closes the specified by ordinal number tab.
Args: num
### !tab next
Opens next tab
Args: -
### !tab prev
Opens previous tab
Args: -
### !tab rename
Renames specified tab.
Args: [num] [new name]
### !tab left
Toggles left area tab. Expiremental feature.
Args: -
### !tab leftuse
Toggles whether left area tab should be edited in insert mode or not. Expiremental.
Args: -
### !tab showdiffn
Opens last edits in current tab. Currently edit history not really working.
Args: -
## !alias
Editor commands aliases manager.
### !alias new
Creates new alias.
Args: alias_name command com_args(optional)
### !alias rm
Removes saved alias.
Args: alias_name
