LET x = 0;
WHILE x < 3 BEGIN
    PRINTLN x;
    LET x = x + 1;
END

// Infinite loop with BREAK (will break at 3)
LET i = 0;
WHILE TRUE BEGIN
    LET i = i + 1;
    IF i == 3 THEN BEGIN // Block IF
        BREAK;
    END
    PRINTLN i;
END

// Using CONTINUE (skip 3)
LET j = 0;
WHILE j < 5 BEGIN
    LET j = j + 1;
    IF j == 3 THEN BEGIN
        CONTINUE;
    END
    PRINTLN j;
END

// Infinite loop with BREAK (will break at 3)
LET i = 0;
WHILE TRUE BEGIN
    LET i = i + 1;
    IF i == 3 THEN BREAK; // Immediate IF
    PRINTLN i;
END

// Using CONTINUE (skip 3)
LET j = 0;
WHILE j < 5 BEGIN
    LET j = j + 1;
    IF j == 3 THEN  CONTINUE;
    PRINTLN j;
END


// FALSE as never-enter condition
WHILE FALSE BEGIN
    PRINTLN "You should never see this";
END
