
import React from "react";

export interface SearchStats {
    keysTested: string;
    keysPerSecond: number;
    keysPerSecondFmt: string;
    probabilityPercent: number;
    estTime50Percent: string;
}

interface StatsDashboardProps {
    stats: SearchStats | null;
    useGpu: boolean;
}

export const StatsDashboard: React.FC<StatsDashboardProps> = ({ stats, useGpu }) => {
    if (!stats) return null;

    return (
        <div className="stats-dashboard">
            <div className="stat-card">
                <div className="stat-icon">{useGpu ? "🚀" : "💻"}</div>
                <div className="stat-content">
                    <div className="stat-value">{stats.keysPerSecondFmt}</div>
                    <div className="stat-label">Current Speed</div>
                </div>
            </div>

            <div className="stat-card">
                <div className="stat-icon">🎲</div>
                <div className="stat-content">
                    <div className="stat-value">{stats.probabilityPercent.toFixed(1)}%</div>
                    <div className="stat-label">Probability (since start)</div>
                </div>
            </div>

            <div className="stat-card">
                <div className="stat-icon">⏱️</div>
                <div className="stat-content">
                    <div className="stat-value">{stats.estTime50Percent}</div>
                    <div className="stat-label">Est. Time (50% Prob)</div>
                </div>
            </div>
        </div>
    );
};
