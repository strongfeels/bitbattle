"use client";

import React, { useEffect, useRef } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, ViewUpdate, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import { javascript } from "@codemirror/lang-javascript";

type Props = {
    value: string;
    readOnly?: boolean;
    onChange?: (val: string) => void;
    className?: string;
    style?: React.CSSProperties;
};

function CodeMirrorEditor({
                              value,
                              readOnly = false,
                              onChange,
                              className,
                              style = { height: "60vh", border: "1px solid #444" }
                          }: Props) {
    const editorDiv = useRef<HTMLDivElement>(null);
    const editorView = useRef<EditorView | null>(null);
    const isUpdatingFromProp = useRef(false);

    useEffect(() => {
        if (!editorDiv.current) return;

        if (editorView.current) {
            editorView.current.destroy();
        }

        const state = EditorState.create({
            doc: value,
            extensions: [
                lineNumbers(),
                keymap.of(defaultKeymap),
                javascript(),
                EditorView.editable.of(!readOnly),
                EditorView.updateListener.of((update: ViewUpdate) => {
                    if (update.docChanged && onChange && !isUpdatingFromProp.current) {
                        const doc = update.state.doc;
                        onChange(doc.toString());
                    }
                }),
                EditorView.theme({
                    "&": {
                        fontSize: "14px",
                        fontFamily: "monospace",
                    },
                    ".cm-editor": {
                        outline: "none",
                        border: "1px solid #ccc",
                    },
                    ".cm-focused": {
                        outline: "none",
                        borderColor: "#007acc",
                    },
                }),
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
    }, [readOnly]);

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
            className={className}
            style={style}
        />
    );
}

export default CodeMirrorEditor;