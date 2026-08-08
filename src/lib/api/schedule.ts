import { invoke } from '@tauri-apps/api/core';
import type { CreateScheduledJobRequest, ScheduledJob, UpdateScheduledJobRequest } from './types';

export function listScheduledJobs(): Promise<ScheduledJob[]> {
  return invoke<ScheduledJob[]>('list_scheduled_jobs');
}

export function createScheduledJob(req: CreateScheduledJobRequest): Promise<ScheduledJob> {
  return invoke<ScheduledJob>('create_scheduled_job', { req });
}

export function updateScheduledJob(req: UpdateScheduledJobRequest): Promise<ScheduledJob | null> {
  return invoke<ScheduledJob | null>('update_scheduled_job', { req });
}

export function deleteScheduledJob(id: string): Promise<void> {
  return invoke<void>('delete_scheduled_job', { id });
}
