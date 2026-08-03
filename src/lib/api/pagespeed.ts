import { invoke } from '@tauri-apps/api/core';
import type { PageSpeedData } from './types';

export function getPagespeedScore(projectId: string, url: string): Promise<PageSpeedData> {
  return invoke<PageSpeedData>('get_pagespeed_score', { projectId, url });
}
