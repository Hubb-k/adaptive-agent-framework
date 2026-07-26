#!/usr/bin/env python3
import csv
import sys
from pathlib import Path

def generate_html(log_file, domain_name="Grid Stability"):
    data = []
    with open(log_file, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            data.append({
                'tick': int(row['tick']),
                'target': float(row['target']),
                'alignment': float(row['alignment']),
                'inertia': float(row['inertia']),
                'population': int(row['population']),
                'hits': int(row['hits']),
                'reward': float(row['reward']),
                'event': row.get('event', '')
            })
    
    if not data:
        print("Нет данных для визуализации")
        return
    
    html = f"""<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <title>{domain_name} Analysis</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 20px;
            background: #1a1a1a;
            color: #e0e0e0;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
        }}
        h1 {{
            color: #4CAF50;
            text-align: center;
        }}
        .chart-container {{
            background: #2a2a2a;
            padding: 20px;
            margin: 20px 0;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.3);
        }}
        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin: 20px 0;
        }}
        .stat-card {{
            background: #2a2a2a;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
        }}
        .stat-value {{
            font-size: 24px;
            font-weight: bold;
            color: #4CAF50;
        }}
        .stat-label {{
            font-size: 14px;
            color: #888;
            margin-top: 5px;
        }}
        canvas {{
            max-height: 400px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>📊 {domain_name} Analysis</h1>
        
        <div class="stats">
            <div class="stat-card">
                <div class="stat-value">{data[-1]['tick']}</div>
                <div class="stat-label">Total Ticks</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{data[-1]['hits']}</div>
                <div class="stat-label">Resonance Hits</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{data[-1]['alignment']:.3f}</div>
                <div class="stat-label">Final Alignment</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{data[-1]['inertia']:.3f}</div>
                <div class="stat-label">Final Inertia</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{data[-1]['population']}</div>
                <div class="stat-label">Final Population</div>
            </div>
        </div>
        
        <div class="chart-container">
            <h2>Target vs Alignment</h2>
            <canvas id="alignmentChart"></canvas>
        </div>
        
        <div class="chart-container">
            <h2>System Inertia Over Time</h2>
            <canvas id="inertiaChart"></canvas>
        </div>
        
        <div class="chart-container">
            <h2>Population & Reward Dynamics</h2>
            <canvas id="populationChart"></canvas>
        </div>
    </div>
    
    <script>
        const data = {data};
        const labels = data.map(d => d.tick);
        
        new Chart(document.getElementById('alignmentChart'), {{
            type: 'line',
            data: {{
                labels: labels,
                datasets: [
                    {{
                        label: 'Target',
                        data: data.map(d => d.target),
                        borderColor: '#FF9800',
                        backgroundColor: 'rgba(255, 152, 0, 0.1)',
                        tension: 0.1,
                        pointRadius: 0
                    }},
                    {{
                        label: 'Alignment',
                        data: data.map(d => d.alignment),
                        borderColor: '#4CAF50',
                        backgroundColor: 'rgba(76, 175, 80, 0.1)',
                        tension: 0.1,
                        pointRadius: 0
                    }}
                ]
            }},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        labels: {{ color: '#e0e0e0' }}
                    }}
                }},
                scales: {{
                    x: {{
                        ticks: {{ color: '#888' }},
                        grid: {{ color: '#333' }}
                    }},
                    y: {{
                        ticks: {{ color: '#888' }},
                        grid: {{ color: '#333' }}
                    }}
                }}
            }}
        }});
        
        new Chart(document.getElementById('inertiaChart'), {{
            type: 'line',
            data: {{
                labels: labels,
                datasets: [{{
                    label: 'Inertia',
                    data: data.map(d => d.inertia),
                    borderColor: '#2196F3',
                    backgroundColor: 'rgba(33, 150, 243, 0.1)',
                    tension: 0.1,
                    pointRadius: 0
                }}]
            }},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        labels: {{ color: '#e0e0e0' }}
                    }}
                }},
                scales: {{
                    x: {{
                        ticks: {{ color: '#888' }},
                        grid: {{ color: '#333' }}
                    }},
                    y: {{
                        ticks: {{ color: '#888' }},
                        grid: {{ color: '#333' }}
                    }}
                }}
            }}
        }});
        
        new Chart(document.getElementById('populationChart'), {{
            type: 'line',
            data: {{
                labels: labels,
                datasets: [
                    {{
                        label: 'Population',
                        data: data.map(d => d.population),
                        borderColor: '#9C27B0',
                        backgroundColor: 'rgba(156, 39, 176, 0.1)',
                        yAxisID: 'y',
                        tension: 0.1,
                        pointRadius: 0
                    }},
                    {{
                        label: 'Reward',
                        data: data.map(d => d.reward),
                        borderColor: '#E91E63',
                        backgroundColor: 'rgba(233, 30, 99, 0.1)',
                        yAxisID: 'y1',
                        tension: 0.1,
                        pointRadius: 0
                    }}
                ]
            }},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        labels: {{ color: '#e0e0e0' }}
                    }}
                }},
                scales: {{
                    x: {{
                        ticks: {{ color: '#888' }},
                        grid: {{ color: '#333' }}
                    }},
                    y: {{
                        type: 'linear',
                        display: true,
                        position: 'left',
                        ticks: {{ color: '#9C27B0' }},
                        grid: {{ color: '#333' }},
                        title: {{
                            display: true,
                            text: 'Population',
                            color: '#9C27B0'
                        }}
                    }},
                    y1: {{
                        type: 'linear',
                        display: true,
                        position: 'right',
                        ticks: {{ color: '#E91E63' }},
                        grid: {{ drawOnChartArea: false }},
                        title: {{
                            display: true,
                            text: 'Reward',
                            color: '#E91E63'
                        }}
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>"""
    
    return html

if __name__ == '__main__':
    log_file = sys.argv[1] if len(sys.argv) > 1 else 'examples/grid_stability.log'
    domain = sys.argv[2] if len(sys.argv) > 2 else 'Grid Stability'
    
    if not Path(log_file).exists():
        print(f"Файл {log_file} не найден")
        sys.exit(1)
    
    print(f"Читаю данные из {log_file}...")
    html = generate_html(log_file, domain)
    
    output_file = f'tools/{Path(log_file).stem}_analysis.html'
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(html)
    
    print(f"✓ Визуализация создана: {output_file}")