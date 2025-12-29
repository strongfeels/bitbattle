"use client";

import React, { useEffect, useRef } from "react";
import { EditorState, Compartment } from "@codemirror/state";
import {
    EditorView,
    ViewUpdate,
    keymap,
    lineNumbers,
    highlightActiveLineGutter,
    highlightSpecialChars,
    drawSelection,
    dropCursor,
    rectangularSelection,
    crosshairCursor,
    highlightActiveLine
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import {
    indentOnInput,
    syntaxHighlighting,
    defaultHighlightStyle,
    bracketMatching,
    foldGutter,
    foldKeymap
} from "@codemirror/language";
import { javascript } from "@codemirror/lang-javascript";
import { python } from "@codemirror/lang-python";
import { java } from "@codemirror/lang-java";
import { cpp } from "@codemirror/lang-cpp";
import { rust } from "@codemirror/lang-rust";
import { go } from "@codemirror/lang-go";
import { oneDark } from "@codemirror/theme-one-dark";
import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { lintKeymap } from "@codemirror/lint";

type Props = {
    value: string;
    readOnly?: boolean;
    onChange?: (val: string) => void;
    className?: string;
    style?: React.CSSProperties;
    language?: string;
    allowPaste?: boolean;
};

// Get language extension based on language string
function getLanguageExtension(language: string) {
    switch (language) {
        case 'python':
            return python();
        case 'java':
            return java();
        case 'c':
        case 'cpp':
            return cpp();
        case 'rust':
            return rust();
        case 'go':
            return go();
        case 'javascript':
        default:
            return javascript();
    }
}

// Custom dark theme that's similar to VS Code / LeetCode
const customTheme = EditorView.theme({
    "&": {
        fontSize: "14px",
        fontFamily: "'JetBrains Mono', 'Fira Code', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace",
        backgroundColor: "#1e1e1e",
    },
    ".cm-content": {
        caretColor: "#ffffff",
        padding: "10px 0",
    },
    ".cm-cursor": {
        borderLeftColor: "#ffffff",
        borderLeftWidth: "2px",
    },
    "&.cm-focused .cm-cursor": {
        borderLeftColor: "#ffffff",
    },
    ".cm-gutters": {
        backgroundColor: "#1e1e1e",
        color: "#858585",
        border: "none",
        borderRight: "1px solid #333",
        paddingRight: "8px",
    },
    ".cm-lineNumbers .cm-gutterElement": {
        padding: "0 8px 0 16px",
        minWidth: "40px",
    },
    ".cm-activeLineGutter": {
        backgroundColor: "#2a2a2a",
        color: "#c6c6c6",
    },
    ".cm-activeLine": {
        backgroundColor: "#2a2a2a40",
    },
    ".cm-selectionBackground": {
        backgroundColor: "#264f78 !important",
    },
    "&.cm-focused .cm-selectionBackground": {
        backgroundColor: "#264f78 !important",
    },
    ".cm-matchingBracket": {
        backgroundColor: "#3a3a3a",
        outline: "1px solid #888",
    },
    ".cm-foldGutter": {
        width: "16px",
    },
    ".cm-foldPlaceholder": {
        backgroundColor: "#3a3a3a",
        border: "none",
        color: "#ddd",
        padding: "0 4px",
        borderRadius: "3px",
    },
    ".cm-tooltip": {
        backgroundColor: "#252526",
        border: "1px solid #454545",
        borderRadius: "4px",
    },
    ".cm-tooltip-autocomplete": {
        "& > ul": {
            fontFamily: "'JetBrains Mono', monospace",
            fontSize: "13px",
        },
        "& > ul > li": {
            padding: "4px 8px",
        },
        "& > ul > li[aria-selected]": {
            backgroundColor: "#094771",
            color: "#ffffff",
        },
    },
    ".cm-scroller": {
        overflow: "auto",
        lineHeight: "1.6",
    },
    ".cm-panels": {
        backgroundColor: "#252526",
        color: "#cccccc",
    },
    ".cm-panels.cm-panels-top": {
        borderBottom: "1px solid #454545",
    },
    ".cm-searchMatch": {
        backgroundColor: "#623315",
        outline: "1px solid #966632",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
        backgroundColor: "#515c6a",
    },
}, { dark: true });

function CodeMirrorEditor({
    value,
    readOnly = false,
    onChange,
    className,
    style = { height: "100%", minHeight: "300px" },
    language = "javascript",
    allowPaste = false
}: Props) {
    const editorDiv = useRef<HTMLDivElement>(null);
    const editorView = useRef<EditorView | null>(null);
    const isUpdatingFromProp = useRef(false);
    const languageCompartment = useRef(new Compartment());

    useEffect(() => {
        if (!editorDiv.current) return;

        if (editorView.current) {
            editorView.current.destroy();
        }

        const state = EditorState.create({
            doc: value,
            extensions: [
                // Line numbers with active line highlighting
                lineNumbers(),
                highlightActiveLineGutter(),
                highlightActiveLine(),

                // Special characters and selection
                highlightSpecialChars(),
                drawSelection(),
                dropCursor(),
                rectangularSelection(),
                crosshairCursor(),

                // History (undo/redo)
                history(),

                // Code folding
                foldGutter({
                    openText: "▼",
                    closedText: "▶",
                }),

                // Indentation
                indentOnInput(),

                // Syntax highlighting
                syntaxHighlighting(defaultHighlightStyle, { fallback: true }),

                // Bracket matching and auto-close
                bracketMatching(),
                closeBrackets(),

                // Autocomplete
                autocompletion({
                    activateOnTyping: true,
                    maxRenderedOptions: 10,
                }),

                // Search highlighting
                highlightSelectionMatches(),

                // Language support (in compartment for dynamic switching)
                languageCompartment.current.of(getLanguageExtension(language)),

                // Keymaps
                keymap.of([
                    ...closeBracketsKeymap,
                    ...defaultKeymap,
                    ...searchKeymap,
                    ...historyKeymap,
                    ...foldKeymap,
                    ...completionKeymap,
                    ...lintKeymap,
                    indentWithTab,
                ]),

                // Editable state
                EditorView.editable.of(!readOnly),

                // Update listener
                EditorView.updateListener.of((update: ViewUpdate) => {
                    if (update.docChanged && onChange && !isUpdatingFromProp.current) {
                        onChange(update.state.doc.toString());
                    }
                }),

                // Theme
                oneDark,
                customTheme,

                // Tab size
                EditorState.tabSize.of(4),

                // Paste prevention (when allowPaste is false)
                ...(allowPaste ? [] : [
                    EditorView.domEventHandlers({
                        paste(event) {
                            event.preventDefault();
                            // Could show a toast/notification here
                            console.log('Pasting is disabled in BitBattle');
                            return true;
                        },
                        drop(event) {
                            // Also prevent drag-and-drop of text
                            event.preventDefault();
                            return true;
                        }
                    })
                ]),
            ],
        });

        editorView.current = new EditorView({
            state,
            parent: editorDiv.current,
        });

        return () => {
            if (editorView.current) {
                editorView.current.destroy();
                editorView.current = null;
            }
        };
    }, [readOnly, allowPaste]);

    // Update language when it changes
    useEffect(() => {
        if (!editorView.current) return;

        editorView.current.dispatch({
            effects: languageCompartment.current.reconfigure(getLanguageExtension(language))
        });
    }, [language]);

    // Update content when value prop changes
    useEffect(() => {
        if (!editorView.current) return;

        const currentValue = editorView.current.state.doc.toString();
        if (value !== currentValue) {
            isUpdatingFromProp.current = true;

            editorView.current.dispatch({
                changes: {
                    from: 0,
                    to: currentValue.length,
                    insert: value
                },
            });

            setTimeout(() => {
                isUpdatingFromProp.current = false;
            }, 0);
        }
    }, [value]);

    return (
        <div
            ref={editorDiv}
            className={`overflow-hidden rounded ${className || ''}`}
            style={style}
        />
    );
}

export default CodeMirrorEditor;
