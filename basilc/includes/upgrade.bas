' upgrade.bas
' Interactive upgrader / installer for BASIC or BASIL components.

CONST OS_WINDOWS = "W"
CONST OS_LINUX   = "L"
CONST OS_MAC     = "M"

CONST ITEM_BASIL    = "B"
CONST ITEM_COMPILER = "C"
CONST ITEM_WEBSERVER= "W"
CONST ITEM_ALL      = "A"

SUB Main()
    DIM osChoice$, itemChoice$, targetDir$
    DIM ok%

    PRINT "=== Basil / BASIC Upgrade Utility ==="
    PRINT

    osChoice$ = PromptChoice$("Select OS (W)indows (L)inux or (M)ac <L> : ", OS_LINUX, "WLM")
    itemChoice$ = PromptChoice$("Select item(s) to download/upgrade (B)asil, (C)ompiler, (W)ebserver, (A)ll <A> : ", ITEM_ALL, "BCWA")

    targetDir$ = DefaultTargetDir$(osChoice$)
    targetDir$ = PromptWithDefault$("Enter path for binaries <" + targetDir$ + "> : ", targetDir$)

    targetDir$ = Trim$(targetDir$)
    IF targetDir$ = "" THEN
        PRINT "No target directory specified. Aborting."
        EXIT SUB
    END IF

    PRINT
    PRINT "Summary:"
    PRINT "  OS:        "; DescribeOS$(osChoice$)
    PRINT "  Items:     "; DescribeItems$(itemChoice$)
    PRINT "  Target dir:"; targetDir$
    PRINT

    PRINT "Press ENTER to download files, or 'c' to abort: ";
    DIM k$
    LINE INPUT k$
    IF k$ <> "" AND (UCASE$(LEFT$(k$, 1)) = "C") THEN
        PRINT "Aborted by user."
        EXIT SUB
    END IF

    ok% = PerformUpgrade%(osChoice$, itemChoice$, targetDir$)
    IF ok% = 0 THEN
        PRINT
        PRINT "Upgrade completed successfully."
    ELSE
        PRINT
        PRINT "Upgrade finished with errors. One or more components may have failed."
    END IF

END SUB


FUNCTION PromptWithDefault$(prompt$, def$) AS STRING
    DIM line$
    PRINT prompt;
    LINE INPUT line$
    line$ = Trim$(line$)
    IF line$ = "" THEN
        PromptWithDefault$ = def$
    ELSE
        PromptWithDefault$ = line$
    END IF
END FUNCTION


FUNCTION PromptChoice$(prompt$, def$, valid$) AS STRING
    DIM line$, ch$
    DO
        PRINT prompt;
        LINE INPUT line$
        line$ = Trim$(line$)
        IF line$ = "" THEN
            ch$ = UCASE$(def$)
        ELSE
            ch$ = UCASE$(LEFT$(line$, 1))
        END IF

        IF InSet$(ch$, valid$) THEN
            PromptChoice$ = ch$
            EXIT FUNCTION
        ELSE
            PRINT "Invalid choice. Please enter one of: "; valid$
        END IF
    LOOP
END FUNCTION


FUNCTION InSet$(value$, valid$) AS INTEGER
    DIM i%
    FOR i% = 1 TO LEN(valid$)
        IF MID$(valid$, i%, 1) = value$ THEN
            InSet$ = -1
            EXIT FUNCTION
        END IF
    NEXT
    InSet$ = 0
END FUNCTION


FUNCTION Trim$(s$) AS STRING
    DIM i%, j%
    i% = 1
    WHILE i% <= LEN(s$) AND MID$(s$, i%, 1) <= " "
        i% = i% + 1
    WEND

    j% = LEN(s$)
    WHILE j% >= i% AND MID$(s$, j%, 1) <= " "
        j% = j% - 1
    WEND

    IF j% < i% THEN
        Trim$ = ""
    ELSE
        Trim$ = MID$(s$, i%, j% - i% + 1)
    END IF
END FUNCTION


FUNCTION DescribeOS$(os$) AS STRING
    SELECT CASE os$
        CASE OS_WINDOWS
            DescribeOS$ = "Windows"
        CASE OS_LINUX
            DescribeOS$ = "Linux"
        CASE OS_MAC
            DescribeOS$ = "macOS"
        CASE ELSE
            DescribeOS$ = "Unknown"
    END SELECT
END FUNCTION


FUNCTION DescribeItems$(item$) AS STRING
    SELECT CASE item$
        CASE ITEM_BASIL
            DescribeItems$ = "Basil / Basic interpreter only"
        CASE ITEM_COMPILER
            DescribeItems$ = "Compiler only"
        CASE ITEM_WEBSERVER
            DescribeItems$ = "Web server only"
        CASE ITEM_ALL
            DescribeItems$ = "All components"
        CASE ELSE
            DescribeItems$ = "Unknown set"
    END SELECT
END FUNCTION


FUNCTION DefaultTargetDir$(os$) AS STRING
    DIM p$
    ' Try to use the path of the current executable if available.
    p$ = EXEPATH$()

    IF Trim$(p$) <> "" THEN
        DefaultTargetDir$ = p$
        EXIT FUNCTION
    END IF

    SELECT CASE os$
        CASE OS_WINDOWS
            DefaultTargetDir$ = "C:\Program Files\Basil"
        CASE OS_LINUX
            DefaultTargetDir$ = "/usr/local/bin"
        CASE OS_MAC
            DefaultTargetDir$ = "/usr/local/bin"
        CASE ELSE
            DefaultTargetDir$ = "."
    END SELECT
END FUNCTION


FUNCTION PerformUpgrade%(os$, item$, targetDir$) AS INTEGER
    DIM failures%

    IF item$ = ITEM_BASIL OR item$ = ITEM_ALL THEN
        IF UpgradeComponent%(os$, "basil", targetDir$) <> 0 THEN
            failures% = failures% + 1
        END IF
    END IF

    IF item$ = ITEM_COMPILER OR item$ = ITEM_ALL THEN
        IF UpgradeComponent%(os$, "compiler", targetDir$) <> 0 THEN
            failures% = failures% + 1
        END IF
    END IF

    IF item$ = ITEM_WEBSERVER OR item$ = ITEM_ALL THEN
        IF UpgradeComponent%(os$, "webserver", targetDir$) <> 0 THEN
            failures% = failures% + 1
        END IF
    END IF

    IF failures% = 0 THEN
        PerformUpgrade% = 0
    ELSE
        PerformUpgrade% = failures%
    END IF
END FUNCTION


FUNCTION UpgradeComponent%(os$, component$, targetDir$) AS INTEGER
    DIM url$, filename$, destPath$, rc%

    filename$ = ComponentFilename$(os$, component$)
    IF filename$ = "" THEN
        PRINT "Skipping "; component$; " (no filename defined for this OS)."
        UpgradeComponent% = 1
        EXIT FUNCTION
    END IF

    url$ = ComponentURL$(os$, component$, filename$)
    destPath$ = JoinPath$(targetDir$, filename$)

    PRINT
    PRINT "Downloading "; component$; " from:"
    PRINT "  "; url$
    PRINT "to:"
    PRINT "  "; destPath$

    rc% = NET_DOWNLOAD_FILE%(url$, destPath$)
    IF rc% <> 0 THEN
        PRINT "  ERROR: Download failed with code "; rc%
        UpgradeComponent% = 1
        EXIT FUNCTION
    END IF

    ' Mark as executable on Unix-like systems.
    IF os$ = OS_LINUX OR os$ = OS_MAC THEN
        MakeExecutable destPath$
    END IF

    PRINT "  OK"
    UpgradeComponent% = 0
END FUNCTION


FUNCTION ComponentFilename$(os$, component$) AS STRING
    DIM base$

    SELECT CASE component$
        CASE "basil"
            base$ = "basilc"
        CASE "compiler"
            base$ = "bcc"
        CASE "webserver"
            base$ = "basil-serve"
        CASE ELSE
            base$ = ""
    END SELECT

    IF base$ = "" THEN
        ComponentFilename$ = ""
        EXIT FUNCTION
    END IF

    SELECT CASE os$
        CASE OS_WINDOWS
            ComponentFilename$ = base$ + ".exe"
        CASE ELSE
            ComponentFilename$ = base$
    END SELECT
END FUNCTION


FUNCTION ComponentURL$(os$, component$, filename$) AS STRING
    ' TODO: Replace this base URL with your actual download endpoint.
    DIM base$

    ' Example layout:
    '   https://basilbasic.com/downloads/<os>/<filename>
    SELECT CASE os$
        CASE OS_WINDOWS
            base$ = "https://basilbasic.com/downloads/windows/"
        CASE OS_LINUX
            base$ = "https://basilbasic.com/downloads/linux/"
        CASE OS_MAC
            base$ = "https://basilbasic.com/downloads/macos/"
        CASE ELSE
            base$ = "https://basilbasic.com/downloads/"
    END SELECT

    IF RIGHT$(base$, 1) <> "/" THEN
        base$ = base$ + "/"
    END IF

    ComponentURL$ = base$ + filename$
END FUNCTION


SUB MakeExecutable(path$)
    ' Optional: make the binary executable on Unix-like systems.
    ' This assumes a SHELL built-in or similar capability exists.
    DIM cmd$
    cmd$ = "chmod +x " + QuotePath$(path$)
    ' If your BASIC has SHELL, you can enable this:
    ' SHELL cmd$
END SUB


FUNCTION JoinPath$(dir$, file$) AS STRING
    IF dir$ = "" THEN
        JoinPath$ = file$
    ELSEIF RIGHT$(dir$, 1) = "/" OR RIGHT$(dir$, 1) = "\" THEN
        JoinPath$ = dir$ + file$
    ELSE
        ' Simple heuristic: use "/" here; OS can normalize if needed.
        JoinPath$ = dir$ + "/" + file$
    END IF
END FUNCTION


FUNCTION QuotePath$(s$) AS STRING
    ' Very simple quoting; you may want to improve this for Windows.
    QuotePath$ = "'" + s$ + "'"
END FUNCTION


' --- Program entry point ---
CALL Main()
