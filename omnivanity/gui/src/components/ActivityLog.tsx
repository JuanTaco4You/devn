
import React, { useRef, useEffect } from "react";

export interface LogEntry {
    timestamp: string;
    level: "info" | "success" | "error";
    message: string;
}

interface ActivityLogProps {
    logs: LogEntry[];
}

export const ActivityLog: React.FC<ActivityLogProps> = ({ logs }) => {
    const endRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        endRef.current?.scrollIntoView({ behavior: "smooth" });
    }, [logs]);

    return (
        <div className="activity-log">
            {logs.map((log, index) => (
                <div key={index} className={`log-entry ${log.level}`}>
                    <span className="log-time">[{log.timestamp}]</span>
                    <span className="log-message">{log.message}</span>
                </div>
            ))}
            <div ref={endRef} />
        </div>
    );
};
