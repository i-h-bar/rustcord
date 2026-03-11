from uuid import UUID


def increment_uuid(uuid: str) -> str:
    uuid_str = UUID(uuid)
    uuid_int = int(uuid_str) + 1

    return str(UUID(int=uuid_int))
