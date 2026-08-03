import {
  Chart,
  ArcElement,
  BarElement,
  BarController,
  CategoryScale,
  DoughnutController,
  Legend,
  LinearScale,
  Tooltip,
} from 'chart.js';

Chart.register(
  DoughnutController,
  BarController,
  ArcElement,
  BarElement,
  CategoryScale,
  LinearScale,
  Legend,
  Tooltip
);

export { Chart };
export type { ChartConfiguration, ChartType, Plugin } from 'chart.js';
