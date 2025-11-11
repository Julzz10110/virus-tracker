import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../store/authStore';

const DashboardView = () => import('../views/DashboardView.vue');
const VariantsView = () => import('../views/VariantsView.vue');
const AnalysisView = () => import('../views/AnalysisView.vue');
const PhylogenyView = () => import('../views/PhylogenyView.vue');
const LoginView = () => import('../views/LoginView.vue');
const AdminUsersView = () => import('../views/AdminUsersView.vue');

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView,
    },
    {
      path: '/variants',
      name: 'variants',
      component: VariantsView,
    },
    {
      path: '/analysis',
      name: 'analysis',
      component: AnalysisView,
    },
    {
      path: '/phylogeny',
      name: 'phylogeny',
      component: PhylogenyView,
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView,
    },
    {
      path: '/admin/users',
      name: 'admin-users',
      component: AdminUsersView,
      meta: { requiresAuth: true, roles: ['admin'] },
    },
  ],
});

router.beforeEach(async (to) => {
  const store = useAuthStore();
  if (to.meta?.requiresAuth) {
    if (!store.isAuthenticated) {
      return { path: '/login', query: { redirect: to.fullPath } };
    }
    const requiredRoles = to.meta.roles || [];
    if (requiredRoles.length && !requiredRoles.some((role) => store.roles.includes(role))) {
      return { path: '/' };
    }
  }
  if (to.path === '/login' && store.isAuthenticated) {
    return { path: '/' };
  }
  return true;
});

export default router;


