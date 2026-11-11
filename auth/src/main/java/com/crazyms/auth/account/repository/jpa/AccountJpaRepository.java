package com.crazyms.auth.account.repository.jpa;

import jakarta.persistence.LockModeType;
import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Lock;
import org.springframework.data.jpa.repository.Modifying;
import org.springframework.data.jpa.repository.Query;

public interface AccountJpaRepository extends JpaRepository<AccountEntity, UUID> {

  Optional<AccountEntity> findByEmail(String email);

  // ORDER BY makes Postgres take the row locks in id order, so two callers never deadlock.
  @Lock(LockModeType.PESSIMISTIC_WRITE)
  @Query("SELECT a FROM AccountEntity a WHERE a.id IN :ids ORDER BY a.id")
  List<AccountEntity> findAllByIdForUpdate(Collection<UUID> ids);

  @Modifying
  @Query("UPDATE AccountEntity a SET a.role = :role WHERE a.id = :id")
  int updateRole(UUID id, String role);
}
