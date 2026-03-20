import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'sessions',
      component: () => import('../views/SessionsView.vue'),
    },
    {
      path: '/session/:sessionId',
      name: 'session-detail',
      component: () => import('../views/SessionDetailView.vue'),
      props: true,
    },
    {
      path: '/library',
      name: 'library',
      component: () => import('../views/KnowledgeView.vue'),
    },
  ],
})

export default router
