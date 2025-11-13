Please do this for me.  That is an excellent BASIC program you write, and I want to add some tweaks to BASIC and Basil to support some of the syntaxes you use that we are lacking, in addition to our EXEPATH$() and NET_DOWNLOAD_FILE% functions.

Please create a Junie prompt for me to give to both BASIC Junie and Basil Junie to add EXEPATH$() and NET_DOWNLOAD_FILE% to the language, and also add these things:

CONST VARNAME = "value"

Allow constants to be created and given a string, integer or float value without having to add a type identifier ($ or %).  By default all variables declared in module level code are global within a module (single source file), so let our constants behave the same way, and have the same scope we give to regular variables, but make then immutable.

DIM osChoice$, itemChoice$, targetDir$
DIM ok%

Allow the DIM statement to create empty variables (either null or initialize to 0/blank depending on what works best).  Allow multiple variables to be listed.

This statement doesn't really do anything because variables are automatically created when referenced however I think this is a good plan for a future "Strict" mode or introducing new scopes for variables later one

osChoice$ = PromptChoice$("Select OS (W)indows (L)inux or (M)ac <L> : ", OS_LINUX, "WLM")

Unfortunately we currently require LET when assigning variables, even if they already exist.  I would like to tweak the parser so that LET is implied when a statement begins with a variable assignment like this.  That's another strict-mode thing that we can enforce later like how Javascript will re-scope a variable if it's redeclared with LET.  But for now I just want the keyword to be optional.

    IF targetDir$ = "" THEN
        PRINT "No target directory specified. Aborting."
        EXIT SUB
    END IF

Unfortunately, right now we require this:

    IF targetDir$ = "" THEN BEGIN 
        PRINT "No target directory specified. Aborting."
        EXIT SUB
    END IF

or

    IF targetDir$ = "" THEN 
      BEGIN 
        PRINT "No target directory specified. Aborting."
        EXIT SUB
    END IF

For any multi-line IF-THEN block and for other blocks.  I would like to make "BEGIN" optional if possible but I'm worried that it's really baked into the cake right now.  This would have to be a change that would require a lot of testing to make sure it doesn't break anything.  BEGIN is currently used at the start of every multi-line block including SUB and FUNCTION but I would really like it to be optional.  All code blocks already finish with an END keyword (END SUB, END IF, END WHILE, etc) except for FOREACH .. BEGIN .. NEXT and FOR .. BEGIN .. NEXT.    So this might be tricky and not a hill to die on.  BEGIN is actually synomymous with { and END is synonymous with } and ..SUB ..IF ..WHILE after the END is actually superflous.  It might be a tall order to change this around and make both old a new syntaxes work.